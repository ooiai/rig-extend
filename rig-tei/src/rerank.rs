use rig::http_client::{self, HttpClientExt};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::client::Client;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RerankResult {
    pub index: usize,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(alias = "score", alias = "relevance_score")]
    pub relevance_score: f32,
}

#[derive(thiserror::Error, Debug)]
pub enum RerankError {
    #[error("http error: {0}")]
    Http(#[from] http_client::Error),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("response error: {0}")]
    Response(String),
}

impl Client<reqwest::Client> {
    /// Rerank endpoint (customizable via ClientBuilder): POST {endpoints.rerank}
    pub async fn rerank(
        &self,
        query: &str,
        texts: impl IntoIterator<Item = String>,
        top_n: Option<usize>,
    ) -> Result<Vec<RerankResult>, RerankError> {
        let texts: Vec<String> = texts.into_iter().collect();

        let mut payload = json!({
            "query": query,
            "texts": texts,
        });
        if let Some(k) = top_n {
            payload["top_n"] = json!(k);
        }

        let body =
            serde_json::to_vec(&payload).map_err(|e| RerankError::Response(e.to_string()))?;

        let req = self
            .post_full(&self.endpoints.rerank)
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|e| RerankError::Http(e.into()))?;

        let response = HttpClientExt::send(&self.http_client, req).await?;
        if !response.status().is_success() {
            let text = http_client::text(response).await?;
            return Err(RerankError::Provider(text));
        }

        let bytes: Vec<u8> = response.into_body().await?;
        let parsed: Vec<RerankResult> = serde_json::from_slice(&bytes).map_err(|e| {
            RerankError::Response(format!("Failed to parse TEI rerank response: {e}"))
        })?;
        Ok(parsed)
    }
}
