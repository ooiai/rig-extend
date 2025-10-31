use rig::embeddings::{self, EmbeddingError};
use rig::http_client::{self, HttpClientExt};
use serde::Deserialize;
use serde_json::{Value, json};

use super::client::Client;

#[derive(Debug, Deserialize)]
struct MultiEmbeddings {
    embeddings: Vec<Vec<f32>>,
}

#[derive(Debug, Deserialize)]
struct SingleEmbedding {
    embeddings: Vec<f32>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EmbeddingResponse {
    Multi(MultiEmbeddings),
    Single(SingleEmbedding),
    Bare(Vec<Vec<f32>>),
}

#[derive(Clone)]
pub struct EmbeddingModel<T = reqwest::Client> {
    pub(crate) client: Client<T>,
    pub model: String,
    ndims: usize,
}

impl<T> EmbeddingModel<T> {
    pub fn new(client: Client<T>, model: &str, ndims: usize) -> Self {
        Self {
            client,
            model: model.to_string(),
            ndims,
        }
    }
}

impl<T> embeddings::EmbeddingModel for EmbeddingModel<T>
where
    T: HttpClientExt + Clone + std::fmt::Debug + Send + 'static,
{
    const MAX_DOCUMENTS: usize = 1024;

    fn ndims(&self) -> usize {
        self.ndims
    }

    async fn embed_texts(
        &self,
        documents: impl IntoIterator<Item = String>,
    ) -> Result<Vec<embeddings::Embedding>, EmbeddingError> {
        let docs: Vec<String> = documents.into_iter().collect();

        let inputs_value: Value = if docs.len() == 1 {
            json!({ "inputs": docs[0] })
        } else {
            json!({ "inputs": docs })
        };

        let body = serde_json::to_vec(&inputs_value)?;

        // Use resolved full endpoint (customizable)
        let req = self
            .client
            .post_full(&self.client.endpoints.embed)
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|e| EmbeddingError::HttpError(e.into()))?;

        let response = HttpClientExt::send(&self.client.http_client, req).await?;

        if !response.status().is_success() {
            let text = http_client::text(response).await?;
            return Err(EmbeddingError::ProviderError(text));
        }

        let bytes: Vec<u8> = response.into_body().await?;
        let parsed: EmbeddingResponse = serde_json::from_slice(&bytes).map_err(|e| {
            EmbeddingError::ResponseError(format!("Failed to parse TEI embeddings: {e}"))
        })?;

        let embeddings: Vec<Vec<f64>> = match parsed {
            EmbeddingResponse::Multi(m) => m
                .embeddings
                .into_iter()
                .map(|v| v.into_iter().map(|x| x as f64).collect())
                .collect(),
            EmbeddingResponse::Single(s) => {
                vec![s.embeddings.into_iter().map(|x| x as f64).collect()]
            }
            EmbeddingResponse::Bare(arr) => arr
                .into_iter()
                .map(|v| v.into_iter().map(|x| x as f64).collect())
                .collect(),
        };

        if embeddings.len() != docs.len() {
            return Err(EmbeddingError::ResponseError(
                "Response data length does not match input length".into(),
            ));
        }

        Ok(embeddings
            .into_iter()
            .zip(docs.into_iter())
            .map(|(vec, document)| embeddings::Embedding { document, vec })
            .collect())
    }
}
