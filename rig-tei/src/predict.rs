use rig::http_client::{self, HttpClientExt};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::client::Client;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LabelScore {
    pub label: String,
    pub score: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PredictResponse {
    pub items: Vec<LabelScore>,
}

#[derive(Debug, Deserialize)]
struct ItemsShape {
    items: Vec<LabelScore>,
}
#[derive(Debug, Deserialize)]
struct PredictionsShape {
    predictions: Vec<LabelScore>,
}
#[derive(Debug, Deserialize)]
struct ArraysShape {
    labels: Vec<String>,
    scores: Vec<f32>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PredictResponseInternal {
    Items(ItemsShape),
    Predictions(PredictionsShape),
    Arrays(ArraysShape),
}

#[derive(thiserror::Error, Debug)]
pub enum PredictError {
    #[error("http error: {0}")]
    Http(#[from] http_client::Error),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("response error: {0}")]
    Response(String),
}

impl Client<reqwest::Client> {
    /// Predict/classify inputs using TEI router endpoint (customizable via ClientBuilder)
    pub async fn predict(
        &self,
        inputs: impl IntoIterator<Item = String>,
    ) -> Result<PredictResponse, PredictError> {
        let inputs_vec: Vec<String> = inputs.into_iter().collect();
        let body_value = if inputs_vec.len() == 1 {
            json!({ "inputs": inputs_vec[0] })
        } else {
            json!({ "inputs": inputs_vec })
        };

        let body =
            serde_json::to_vec(&body_value).map_err(|e| PredictError::Response(e.to_string()))?;

        let req = self
            .post_full(&self.endpoints.predict)
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|e| PredictError::Http(e.into()))?;

        let response = HttpClientExt::send(&self.http_client, req).await?;
        if !response.status().is_success() {
            let text = http_client::text(response).await?;
            return Err(PredictError::Provider(text));
        }

        let bytes: Vec<u8> = response.into_body().await?;
        let internal: PredictResponseInternal = serde_json::from_slice(&bytes).map_err(|e| {
            PredictError::Response(format!("Failed to parse TEI predict response: {e}"))
        })?;

        let items = match internal {
            PredictResponseInternal::Items(x) => x.items,
            PredictResponseInternal::Predictions(x) => x.predictions,
            PredictResponseInternal::Arrays(x) => {
                if x.labels.len() != x.scores.len() {
                    return Err(PredictError::Response(
                        "labels and scores length mismatch".into(),
                    ));
                }
                x.labels
                    .into_iter()
                    .zip(x.scores.into_iter())
                    .map(|(label, score)| LabelScore { label, score })
                    .collect()
            }
        };

        Ok(PredictResponse { items })
    }
}
