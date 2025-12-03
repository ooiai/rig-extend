use rig::embeddings::{self, EmbeddingError};
use rig::http_client::{self, HttpClientExt};
use rig::providers::openai::completion::Usage;
use serde::Deserialize;
use serde_json::json;

use super::client::Client;
use super::types::ApiResponse;

// Model constants (aligned with original)
pub const TEXT_DOUBAO_EMBEDDING: &str = "Doubao-embedding";
pub const TEXT_DOUBAO_EMBEDDING_LARGE: &str = "doubao-embedding-large";

#[derive(Debug, Deserialize)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: Vec<f64>,
    pub index: usize,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Clone)]
pub struct EmbeddingModel<T = reqwest::Client> {
    pub(crate) client: Client<T>,
    pub model: String,
    ndims: usize,
}

impl<T> EmbeddingModel<T> {
    pub fn new(client: Client<T>, model: impl Into<String>, ndims: usize) -> Self {
        Self {
            client,
            model: model.into(),
            ndims,
        }
    }
}

impl<T> embeddings::EmbeddingModel for EmbeddingModel<T>
where
    T: HttpClientExt + Clone + std::fmt::Debug + Send + 'static,
{
    const MAX_DOCUMENTS: usize = 1024;

    type Client = Client<T>;

    fn make(client: &Self::Client, model: impl Into<String>, ndims: Option<usize>) -> Self {
        let model = model.into();
        let dims = ndims.unwrap_or(0);
        Self::new(client.clone(), model, dims)
    }

    fn ndims(&self) -> usize {
        self.ndims
    }

    async fn embed_texts(
        &self,
        documents: impl IntoIterator<Item = String>,
    ) -> Result<Vec<embeddings::Embedding>, EmbeddingError> {
        let documents = documents.into_iter().collect::<Vec<_>>();

        let mut body = json!({
            "model": self.model,
            "input": documents,
        });

        if self.ndims > 0 {
            body["dimensions"] = json!(self.ndims);
        }

        let body = serde_json::to_vec(&body)?;

        let req = self
            .client
            .post("/embeddings")?
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|e| EmbeddingError::HttpError(e.into()))?;

        let response = HttpClientExt::send(&self.client.http_client, req).await?;

        if response.status().is_success() {
            let text = http_client::text(response).await?;
            let parsed: ApiResponse<EmbeddingResponse> = serde_json::from_str(&text)?;

            match parsed {
                ApiResponse::Ok(response) => {
                    if let Some(ref usage) = response.usage {
                        tracing::info!(target: "rig", "Volcengine embedding token usage: {}", usage);
                    }

                    if response.data.len() != documents.len() {
                        return Err(EmbeddingError::ResponseError(
                            "Response data length does not match input length".into(),
                        ));
                    }

                    Ok(response
                        .data
                        .into_iter()
                        .zip(documents.into_iter())
                        .map(|(embedding, document)| embeddings::Embedding {
                            document,
                            vec: embedding.embedding,
                        })
                        .collect())
                }
                ApiResponse::Err(err) => Err(EmbeddingError::ProviderError(err.error.message)),
            }
        } else {
            let text = http_client::text(response).await?;
            Err(EmbeddingError::ProviderError(text))
        }
    }
}
