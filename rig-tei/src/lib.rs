//! Category: mod.rs (top-level module and constant exports)
//! Structure mirrors rig-bedrock:
//! - client.rs: Client and Builder; implements Provider/Verify/Embedding
//! - embedding.rs: Text embeddings
//! - rerank.rs: Text reranking
//! - predict.rs: Classification/prediction

pub mod client;
pub mod embedding;
pub mod predict;
pub mod rerank;

pub use client::{Client, Endpoints};
pub use embedding::EmbeddingModel;
pub use predict::{LabelScore, PredictError, PredictResponse};
pub use rerank::{RerankError, RerankResult};

use rig::impl_conversion_traits;

// Default local TEI base URL
pub const TEI_DEFAULT_BASE_URL: &str = "http://127.0.0.1:8080";

// Keep conversion traits consistent with original single-file version
impl_conversion_traits!(
    AsCompletion,
    AsTranscription,
    AsImageGeneration,
    AsAudioGeneration for Client<T>
);
