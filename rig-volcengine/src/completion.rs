use rig::completion::{self, CompletionError, CompletionRequest};
use rig::http_client;
use rig::message;
use rig::providers::openai;
use rig::providers::openai::completion::Usage;
use rig::streaming::StreamingCompletionResponse;

use serde_json::{Value, json};
use tracing::{Instrument, info_span};

use super::client::Client;
use super::types::{ApiResponse, ToolChoice};

/// Local deep-merge helper to avoid private rig::json_utils.
/// - Merge objects recursively, right overrides left; otherwise returns right.
fn merge(left: Value, right: Value) -> Value {
    match (left, right) {
        (Value::Object(mut a), Value::Object(b)) => {
            for (k, v) in b {
                let merged = match a.remove(&k) {
                    Some(existing) => merge(existing, v),
                    None => v,
                };
                a.insert(k, merged);
            }
            Value::Object(a)
        }
        (_, r) => r,
    }
}

/// Chat completion model: CompletionModel<T>
#[derive(Clone)]
pub struct CompletionModel<T = reqwest::Client> {
    pub(crate) client: Client<T>,
    pub model: String,
}

impl<T> CompletionModel<T> {
    pub fn new(client: Client<T>, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
        }
    }

    pub(crate) fn create_completion_request(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<Value, CompletionError> {
        // Build messages (include context documents if any)
        let mut partial_history = vec![];
        if let Some(docs) = completion_request.normalized_documents() {
            partial_history.push(docs);
        }
        partial_history.extend(completion_request.chat_history);

        // Preamble (system) goes first
        let mut full_history: Vec<openai::Message> = completion_request
            .preamble
            .map_or_else(Vec::new, |preamble| {
                vec![openai::Message::system(&preamble)]
            });

        // Convert user/assistant messages
        full_history.extend(
            partial_history
                .into_iter()
                .map(message::Message::try_into)
                .collect::<Result<Vec<Vec<openai::Message>>, _>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
        );

        let tool_choice = completion_request
            .tool_choice
            .map(ToolChoice::try_from)
            .transpose()?;

        // OpenAI-compatible payload
        let request = if completion_request.tools.is_empty() {
            json!({
                "model": self.model,
                "messages": full_history,
                "temperature": completion_request.temperature,
                "max_tokens": completion_request.max_tokens,
            })
        } else {
            json!({
                "model": self.model,
                "messages": full_history,
                "temperature": completion_request.temperature,
                "max_tokens": completion_request.max_tokens,
                "tools": completion_request.tools.into_iter().map(openai::ToolDefinition::from).collect::<Vec<_>>(),
                "tool_choice": tool_choice,
            })
        };

        Ok(if let Some(params) = completion_request.additional_params {
            merge(request, params)
        } else {
            request
        })
    }
}

impl TryFrom<message::ToolChoice> for ToolChoice {
    type Error = CompletionError;

    fn try_from(value: message::ToolChoice) -> Result<Self, Self::Error> {
        let res = match value {
            message::ToolChoice::None => Self::None,
            message::ToolChoice::Auto => Self::Auto,
            message::ToolChoice::Required => Self::Required,
            choice => {
                return Err(CompletionError::ProviderError(format!(
                    "Unsupported tool choice type: {choice:?}"
                )));
            }
        };

        Ok(res)
    }
}

impl<T> completion::CompletionModel for CompletionModel<T>
where
    T: http_client::HttpClientExt + Clone + Default + Send + 'static,
{
    type Response = openai::CompletionResponse;
    type StreamingResponse = openai::StreamingCompletionResponse;
    type Client = Client<T>;

    fn make(client: &Self::Client, model: impl Into<String>) -> Self {
        Self::new(client.clone(), model)
    }

    async fn completion(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<completion::CompletionResponse<openai::CompletionResponse>, CompletionError> {
        let preamble = completion_request.preamble.clone();
        let request = self.create_completion_request(completion_request)?;

        let span = if tracing::Span::current().is_disabled() {
            info_span!(
                target: "rig::completions",
                "chat",
                gen_ai.operation.name = "chat",
                gen_ai.provider.name = "volcengine",
                gen_ai.request.model = self.model,
                gen_ai.system_instructions = preamble,
                gen_ai.response.id = tracing::field::Empty,
                gen_ai.response.model = tracing::field::Empty,
                gen_ai.usage.output_tokens = tracing::field::Empty,
                gen_ai.usage.input_tokens = tracing::field::Empty,
                gen_ai.input.messages = serde_json::to_string(&request.get("messages").unwrap_or(&json!([]))).unwrap(),
                gen_ai.output.messages = tracing::field::Empty,
            )
        } else {
            tracing::Span::current()
        };

        async move {
            let body = serde_json::to_vec(&request)?;
            let req = self
                .client
                .post("/chat/completions")?
                .header("Content-Type", "application/json")
                .body(body)
                .map_err(|e| CompletionError::HttpError(e.into()))?;

            let response = http_client::HttpClientExt::send(&self.client.http_client, req)
                .await
                .map_err(CompletionError::HttpError)?;

            if response.status().is_success() {
                let t = http_client::text(response).await?;
                tracing::debug!(target: "rig::completions", "Volcengine completion response: {t}");

                match serde_json::from_str::<ApiResponse<openai::CompletionResponse>>(&t)? {
                    ApiResponse::Ok(response) => {
                        let span = tracing::Span::current();
                        span.record("gen_ai.response.id", response.id.clone());
                        span.record("gen_ai.response.model_name", response.model.clone());
                        span.record(
                            "gen_ai.output.messages",
                            serde_json::to_string(&response.choices).unwrap(),
                        );
                        if let Some(Usage {
                            prompt_tokens,
                            total_tokens,
                            ..
                        }) = response.usage
                        {
                            span.record("gen_ai.usage.input_tokens", prompt_tokens);
                            span.record(
                                "gen_ai.usage.output_tokens",
                                total_tokens.saturating_sub(prompt_tokens),
                            );
                        }
                        response.try_into()
                    }
                    ApiResponse::Err(err) => Err(CompletionError::ProviderError(err.error.message)),
                }
            } else {
                let t = http_client::text(response).await?;
                Err(CompletionError::ProviderError(t))
            }
        }
        .instrument(span)
        .await
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<StreamingCompletionResponse<Self::StreamingResponse>, CompletionError> {
        super::streaming::stream_completion(self, request).await
    }
}
