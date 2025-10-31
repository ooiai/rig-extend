pub mod client;
pub mod completion;
pub mod embedding;
pub mod rerank;
pub mod streaming;
pub mod types;

pub use client::Client;
pub use completion::CompletionModel;
pub use embedding::{EmbeddingModel, TEXT_EMBEDDING_V4};
pub use rerank::{RerankError, RerankModel, RerankResult, GTE_RERANK_V2};

use rig::impl_conversion_traits;

// Constants (aligned with original single-file version)
pub const BAILIAN_API_BASE_URL: &str = "https://dashscope.aliyuncs.com/compatible-mode/v1";
pub const QWEN3_MAX: &str = "qwen3-max";

// Keep conversion traits consistent with other providers
impl_conversion_traits!(
    AsTranscription,
    AsImageGeneration,
    AsAudioGeneration for Client<T>
);
