//! Category: rerank.rs (text reranking, DashScope gte-rerank-v2)

use serde::{Deserialize, Serialize};

use super::client::Client;

/// Default DashScope rerank endpoint for gte-rerank-v2
pub const GTE_RERANK_V2: &str = "gte-rerank-v2";
pub const GTE_RERANK_V2_URL: &str =
    "https://dashscope.aliyuncs.com/api/v1/services/rerank/text-rerank/text-rerank/";

#[derive(Debug, Serialize)]
pub struct RerankRequest {
    pub model: String,
    pub input: RerankInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<RerankParameters>,
}

#[derive(Debug, Serialize)]
pub struct RerankInput {
    pub query: String,
    pub documents: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RerankParameters {
    pub return_documents: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_n: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RerankResponse {
    pub output: Option<Output>,
    pub message: Option<String>,
    pub usage: Option<Usage>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub total_tokens: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Output {
    pub results: Vec<ResultItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResultItem {
    pub index: usize,
    pub relevance_score: f64,
    pub document: Option<Document>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Document {
    pub text: String,
}

#[derive(thiserror::Error, Debug)]
pub enum RerankError {
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("http status {0}: {1}")]
    HttpStatus(u16, String),
    #[error("response error: {0}")]
    ResponseError(String),
}

/// Rerank model bound to Bailian client
#[derive(Debug, Clone)]
pub struct RerankModel {
    pub(crate) client: Client<reqwest::Client>,
    pub model: String,
    /// Full endpoint URL (base + model), e.g. ".../text-re-rank/gte-rerank-v2"
    pub endpoint: String,
}

impl RerankModel {
    /// Create a rerank model using the Bailian client.
    /// - `endpoint_base`: optional base URL (defaults to DashScope base)
    /// - final endpoint = endpoint_base + model
    pub fn new(
        client: Client<reqwest::Client>,
        model: impl Into<String>,
        endpoint_base: Option<String>, // base URL, not the full endpoint
    ) -> Self {
        let model = model.into();
        let endpoint = endpoint_base.unwrap_or_else(|| GTE_RERANK_V2_URL.to_string());
        Self {
            client,
            model,
            endpoint,
        }
    }

    /// Rerank the given documents based on the query.
    ///
    /// Returns a Vec<RerankResult>. If top_n is provided, the result will be truncated accordingly.
    pub async fn rerank(
        &self,
        query: &str,
        documents: &[String],
        top_n: Option<usize>,
        return_documents: bool,
    ) -> Result<Vec<RerankResult>, RerankError> {
        if query.trim().is_empty() {
            return Err(RerankError::ValidationError(
                "Query cannot be empty".to_string(),
            ));
        }
        if documents.is_empty() {
            return Err(RerankError::ValidationError(
                "Documents cannot be empty".to_string(),
            ));
        }

        let request = RerankRequest {
            model: self.model.clone(),
            input: RerankInput {
                query: query.to_string(),
                documents: documents.to_vec(),
            },
            parameters: Some(RerankParameters {
                return_documents,
                top_n,
            }),
        };

        let resp = self
            .client
            .http_client
            .post(&self.endpoint)
            .bearer_auth(&self.client.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = resp.status();
        let raw_text = resp.text().await?;
        let resp_json: RerankResponse = serde_json::from_str(&raw_text)
            .map_err(|e| RerankError::ResponseError(e.to_string()))?;

        if status.is_success() {
            if let Some(output) = resp_json.output {
                let mut results: Vec<RerankResult> = output
                    .results
                    .into_iter()
                    .map(|item| RerankResult {
                        index: item.index,
                        relevance_score: item.relevance_score,
                        text: item.document.map(|d| d.text).unwrap_or_default(),
                    })
                    .collect();

                if let Some(n) = top_n {
                    results.truncate(n);
                }

                Ok(results)
            } else {
                Err(RerankError::ResponseError(
                    "No output in response".to_string(),
                ))
            }
        } else {
            Err(RerankError::HttpStatus(
                status.as_u16(),
                resp_json
                    .message
                    .unwrap_or_else(|| "Unknown HTTP error".to_string()),
            ))
        }
    }
}

/// Public result returned by rerank()
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResult {
    pub index: usize,
    pub relevance_score: f64,
    /// Flattened document text (empty if server omitted it)
    #[serde(default)]
    pub text: String,
}
