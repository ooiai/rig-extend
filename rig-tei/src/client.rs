use rig::client::{EmbeddingsClient, ProviderClient, VerifyClient, VerifyError};
use rig::http_client::{self};

use super::TEI_DEFAULT_BASE_URL;
use super::embedding::EmbeddingModel;

/// Provider client: Client<T>
/// Note: base_url is resolved into concrete endpoints during build, so we don't store base_url.
#[derive(Clone, Debug)]
pub struct Client<T = reqwest::Client> {
    pub(crate) http_client: T,
    pub(crate) endpoints: Endpoints,
}

/// Resolved endpoints for TEI features.
#[derive(Clone, Debug)]
pub struct Endpoints {
    pub embed: String,
    pub rerank: String,
    pub predict: String,
}

impl Endpoints {
    pub fn with_base(base_url: &str) -> Self {
        let base = base_url.trim_end_matches('/');
        Self {
            embed: format!("{}/embed", base),
            rerank: format!("{}/rerank", base),
            predict: format!("{}/predict", base),
        }
    }
}

/// Client builder: ClientBuilder<'a, T>
pub struct ClientBuilder<'a, T = reqwest::Client> {
    base_url: &'a str,
    http_client: T,
    // Optional endpoint overrides
    embed_endpoint: Option<&'a str>,
    rerank_endpoint: Option<&'a str>,
    predict_endpoint: Option<&'a str>,
}

impl<'a, T> ClientBuilder<'a, T>
where
    T: Default,
{
    pub fn new() -> Self {
        Self {
            base_url: TEI_DEFAULT_BASE_URL,
            http_client: Default::default(),
            embed_endpoint: None,
            rerank_endpoint: None,
            predict_endpoint: None,
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
            base_url: self.base_url,
            http_client,
            embed_endpoint: self.embed_endpoint,
            rerank_endpoint: self.rerank_endpoint,
            predict_endpoint: self.predict_endpoint,
        }
    }

    // Custom endpoint overrides
    pub fn embed_endpoint(mut self, url: &'a str) -> Self {
        self.embed_endpoint = Some(url);
        self
    }

    pub fn rerank_endpoint(mut self, url: &'a str) -> Self {
        self.rerank_endpoint = Some(url);
        self
    }

    pub fn predict_endpoint(mut self, url: &'a str) -> Self {
        self.predict_endpoint = Some(url);
        self
    }

    pub fn build(self) -> Client<T> {
        let mut endpoints = Endpoints::with_base(self.base_url);
        if let Some(url) = self.embed_endpoint {
            endpoints.embed = url.to_string();
        }
        if let Some(url) = self.rerank_endpoint {
            endpoints.rerank = url.to_string();
        }
        if let Some(url) = self.predict_endpoint {
            endpoints.predict = url.to_string();
        }

        Client {
            http_client: self.http_client,
            endpoints,
        }
    }
}

impl<T> Default for Client<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Client<T>
where
    T: Default,
{
    pub fn builder<'a>() -> ClientBuilder<'a, T> {
        ClientBuilder::new()
    }

    pub fn new() -> Self {
        Self::builder().build()
    }
}

// Build a POST request using a full URL (used when endpoints are overridden).
impl<T> Client<T> {
    pub(crate) fn post_full(&self, url: &str) -> http_client::Builder {
        http_client::Builder::new()
            .method(http_client::Method::POST)
            .uri(url.to_string())
    }
}

impl ProviderClient for Client<reqwest::Client> {
    fn from_env() -> Self {
        let base_url =
            std::env::var("TEI_BASE_URL").unwrap_or_else(|_| TEI_DEFAULT_BASE_URL.to_string());
        Self::builder().base_url(&base_url).build()
    }

    fn from_val(input: rig::client::ProviderValue) -> Self {
        let rig::client::ProviderValue::Simple(base_url) = input else {
            panic!("Incorrect provider value type")
        };
        ClientBuilder::new().base_url(&base_url).build()
    }
}

impl VerifyClient for Client<reqwest::Client> {
    async fn verify(&self) -> Result<(), VerifyError> {
        // TEI local router often has no auth and no health endpoint needed.
        Ok(())
    }
}

impl EmbeddingsClient for Client<reqwest::Client> {
    type EmbeddingModel = EmbeddingModel<reqwest::Client>;

    fn embedding_model(&self, model: &str) -> Self::EmbeddingModel {
        EmbeddingModel::new(self.clone(), model, 0)
    }

    fn embedding_model_with_ndims(&self, model: &str, ndims: usize) -> Self::EmbeddingModel {
        EmbeddingModel::new(self.clone(), model, ndims)
    }
}
