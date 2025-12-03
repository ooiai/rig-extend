use rig::client::{CompletionClient, EmbeddingsClient, ProviderClient, VerifyClient, VerifyError};
use rig::http_client::{self, HttpClientExt};

use super::VOLCENGINE_API_BASE_URL;
use super::completion::CompletionModel;
use super::embedding::EmbeddingModel;

/// Provider client: Client<T>
#[derive(Clone)]
pub struct Client<T = reqwest::Client> {
    pub(crate) base_url: String,
    pub(crate) api_key: String,
    pub(crate) http_client: T,
}

impl<T> std::fmt::Debug for Client<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base_url)
            .field("http_client", &self.http_client)
            .field("api_key", &"<REDACTED>")
            .finish()
    }
}

/// Client builder: ClientBuilder<'a, T>
#[derive(Clone)]
pub struct ClientBuilder<'a, T = reqwest::Client> {
    api_key: &'a str,
    base_url: &'a str,
    http_client: T,
}

impl<'a, T> ClientBuilder<'a, T>
where
    T: Default,
{
    pub fn new(api_key: &'a str) -> Self {
        Self {
            api_key,
            base_url: VOLCENGINE_API_BASE_URL,
            http_client: Default::default(),
        }
    }
}

impl<'a, T> ClientBuilder<'a, T> {
    pub fn base_url(mut self, base_url: &'a str) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn with_client<U>(self, http_client: U) -> ClientBuilder<'a, U> {
        ClientBuilder {
            api_key: self.api_key,
            base_url: self.base_url,
            http_client,
        }
    }

    pub fn build(self) -> Client<T> {
        Client {
            base_url: self.base_url.to_string(),
            api_key: self.api_key.to_string(),
            http_client: self.http_client,
        }
    }
}

impl<T> Client<T>
where
    T: Default,
{
    pub fn builder(api_key: &str) -> ClientBuilder<'_, T> {
        ClientBuilder::new(api_key)
    }

    pub fn new(api_key: &str) -> Self {
        Self::builder(api_key).build()
    }
}

impl<T> Client<T>
where
    T: HttpClientExt,
{
    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }

    fn req(
        &self,
        method: http_client::Method,
        path: &str,
    ) -> http_client::Result<http_client::Builder> {
        let url = self.url(path);
        http_client::with_bearer_auth(
            http_client::Builder::new().method(method).uri(url),
            &self.api_key,
        )
    }

    pub(crate) fn get(&self, path: &str) -> http_client::Result<http_client::Builder> {
        self.req(http_client::Method::GET, path)
    }

    pub(crate) fn post(&self, path: &str) -> http_client::Result<http_client::Builder> {
        self.req(http_client::Method::POST, path)
    }
}

impl ProviderClient for Client<reqwest::Client> {
    type Input = String;

    fn from_env() -> Self {
        let api_key = std::env::var("VOLCENGINE_API_KEY").expect("VOLCENGINE_API_KEY not set");
        let base_url = std::env::var("VOLCENGINE_BASE_URL")
            .ok()
            .unwrap_or_else(|| VOLCENGINE_API_BASE_URL.to_string());
        Self::builder(&api_key).base_url(&base_url).build()
    }

    fn from_val(input: String) -> Self {
        Self::new(&input)
    }
}

impl CompletionClient for Client<reqwest::Client> {
    type CompletionModel = CompletionModel<reqwest::Client>;

    fn completion_model(&self, model: impl Into<String>) -> Self::CompletionModel {
        CompletionModel::new(self.clone(), &model.into())
    }
}

impl EmbeddingsClient for Client<reqwest::Client> {
    type EmbeddingModel = EmbeddingModel<reqwest::Client>;

    fn embedding_model(&self, model: impl Into<String>) -> Self::EmbeddingModel {
        EmbeddingModel::new(self.clone(), &model.into(), 0)
    }

    fn embedding_model_with_ndims(
        &self,
        model: impl Into<String>,
        ndims: usize,
    ) -> Self::EmbeddingModel {
        EmbeddingModel::new(self.clone(), &model.into(), ndims)
    }
}

impl VerifyClient for Client<reqwest::Client> {
    async fn verify(&self) -> Result<(), VerifyError> {
        let req = self
            .get("/models")?
            .body(rig::http_client::NoBody)
            .map_err(rig::http_client::Error::from)?;

        let response = HttpClientExt::send(&self.http_client, req).await?;

        match response.status() {
            reqwest::StatusCode::OK => Ok(()),
            reqwest::StatusCode::UNAUTHORIZED => Err(VerifyError::InvalidAuthentication),
            reqwest::StatusCode::INTERNAL_SERVER_ERROR
            | reqwest::StatusCode::SERVICE_UNAVAILABLE
            | reqwest::StatusCode::BAD_GATEWAY => {
                let text = rig::http_client::text(response).await?;
                Err(VerifyError::ProviderError(text))
            }
            _ => Ok(()),
        }
    }
}
