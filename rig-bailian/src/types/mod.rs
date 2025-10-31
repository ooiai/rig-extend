use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub error: BailianError,
}

#[derive(Debug, Deserialize)]
pub struct BailianError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Ok(T),
    Err(ApiErrorResponse),
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
pub enum ToolChoice {
    None,
    #[default]
    Auto,
    Required,
}
