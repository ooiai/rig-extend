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

pub(crate) async fn stream_completion<T>(
    model: &CompletionModel<T>,
    request: CompletionRequest,
) -> Result<
    StreamingCompletionResponse<
        <CompletionModel<T> as rig::completion::CompletionModel>::StreamingResponse,
    >,
    CompletionError,
>
where
    T: rig::http_client::HttpClientExt + Clone + Default + Send + 'static,
{
    let preamble = request.preamble.clone();
    let mut request = model.create_completion_request(request)?;

    // Bailian chat streaming: OpenAI-compatible flags
    request = merge(
        request,
        json!({"stream": true, "stream_options": {"include_usage": true}}),
    );

    let req = model
        .client
        .post("/chat/completions")?
        .header("Content-Type", "application/json")
        .body(serde_json::to_vec(&request)?)
        .map_err(|e| CompletionError::HttpError(e.into()))?;

    let span = if tracing::Span::current().is_disabled() {
        info_span!(
            target: "rig::completions",
            "chat_streaming",
            gen_ai.operation.name = "chat_streaming",
            gen_ai.provider.name = "bailian",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Client;
    use futures::StreamExt;
    use rig::completion::CompletionRequestBuilder;
    use rig::streaming::StreamedAssistantContent;
    use std::time::Duration;
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn stream_sends_json_content_type() {
        tokio::time::timeout(Duration::from_secs(5), async {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let endpoint = format!(
                "http://{}/compatible-mode/v1",
                listener.local_addr().unwrap()
            );
            let client = Client::<reqwest::Client>::builder("test-key")
                .base_url(&endpoint)
                .with_client(reqwest::Client::builder().no_proxy().build().unwrap())
                .build();

            let server = async {
                let (socket, _) = listener.accept().await.unwrap();
                let mut socket = BufReader::new(socket);
                let mut head = String::new();
                while !head.ends_with("\r\n\r\n") {
                    assert_ne!(socket.read_line(&mut head).await.unwrap(), 0);
                }
                assert!(head.starts_with(
                    "POST /compatible-mode/v1/chat/completions HTTP/1.1\r\n"
                ));
                let headers = head.to_ascii_lowercase();
                assert!(headers
                    .lines()
                    .any(|line| line == "content-type: application/json"));
                assert!(headers
                    .lines()
                    .any(|line| line == "authorization: bearer test-key"));
                let content_length: usize = headers
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length: "))
                    .unwrap()
                    .parse()
                    .unwrap();
                let mut body = vec![0; content_length];
                socket.read_exact(&mut body).await.unwrap();
                let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
                assert_eq!(body["model"], "test-model");
                assert_eq!(body["stream"], true);
                assert_eq!(body["stream_options"]["include_usage"], true);

                let events = concat!(
                    "data: {\"choices\":[{\"delta\":{\"content\":\"pong\"}}]}\n\n",
                    "data: {\"choices\":[],\"usage\":{\"prompt_tokens\":1,",
                    "\"completion_tokens\":1,\"total_tokens\":2}}\n\n",
                    "data: [DONE]\n\n"
                );
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    events.len(),
                    events
                );
                socket.get_mut().write_all(response.as_bytes()).await.unwrap();
            };
            let request = async {
                let model = CompletionModel::new(client, "test-model");
                let mut stream = CompletionRequestBuilder::new(model, "ping")
                    .stream()
                    .await
                    .unwrap();
                let mut answer = String::new();
                while let Some(item) = stream.next().await {
                    if let StreamedAssistantContent::Text(text) = item.unwrap() {
                        answer.push_str(&text.text);
                    }
                }
                assert_eq!(answer, "pong");
                assert!(stream.response.is_some());
            };
            tokio::join!(server, request);
        })
        .await
        .expect("local streaming request timed out");
    }
}
