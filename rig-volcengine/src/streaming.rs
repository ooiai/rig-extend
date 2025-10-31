use rig::completion::{CompletionError, CompletionRequest};
use rig::providers::openai::send_compatible_streaming_request;
use rig::streaming::StreamingCompletionResponse;
use serde_json::json;
use tracing::info_span;

use super::completion::CompletionModel;

/// Local deep-merge helper (same rule as in completion.rs)
fn merge(left: serde_json::Value, right: serde_json::Value) -> serde_json::Value {
    match (left, right) {
        (serde_json::Value::Object(mut a), serde_json::Value::Object(b)) => {
            for (k, v) in b {
                let merged = match a.remove(&k) {
                    Some(existing) => merge(existing, v),
                    None => v,
                };
                a.insert(k, merged);
            }
            serde_json::Value::Object(a)
        }
        (_, r) => r,
    }
}

pub(crate) async fn stream_completion(
    model: &CompletionModel<reqwest::Client>,
    request: CompletionRequest,
) -> Result<
    StreamingCompletionResponse<
        <CompletionModel<reqwest::Client> as rig::completion::CompletionModel>::StreamingResponse,
    >,
    CompletionError,
> {
    let preamble = request.preamble.clone();
    let mut request = model.create_completion_request(request)?;

    // Ark chat streaming: OpenAI-compatible flags
    request = merge(
        request,
        json!({"stream": true, "stream_options": {"include_usage": true}}),
    );

    let req = model
        .client
        .post("/chat/completions")?
        .body(serde_json::to_vec(&request)?)
        .map_err(|e| CompletionError::HttpError(e.into()))?;

    let span = if tracing::Span::current().is_disabled() {
        info_span!(
            target: "rig::completions",
            "chat_streaming",
            gen_ai.operation.name = "chat_streaming",
            gen_ai.provider.name = "volcengine",
            gen_ai.request.model = model.model,
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

    tracing::Instrument::instrument(
        send_compatible_streaming_request(model.client.http_client.clone(), req),
        span,
    )
    .await
}
