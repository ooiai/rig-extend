//! Category: mod.rs (top-level module and constant exports)
//! Structure mirrors rig-bedrock:
//! - client.rs: Client and Builder; implements Provider/Verify/Completion/Embedding
//! - completion.rs: Chat completion model (OpenAI-compatible)
//! - embedding.rs: Text embeddings (OpenAI-compatible)
//! - streaming.rs: Streaming (OpenAI-compatible, same flags)
//! - types/mod.rs: Internal shared types (error response, tool choice mapping)

pub mod client;
pub mod completion;
pub mod embedding;
pub mod streaming;
pub mod types;

pub use client::Client;
pub use completion::CompletionModel;
pub use embedding::{EmbeddingModel, TEXT_EMBEDDING_V4};

use rig::impl_conversion_traits;

// Constants (aligned with original single-file version)
pub const BAILIAN_API_BASE_URL: &str = "https://dashscope.aliyuncs.com/compatible-mode/v1"; // keep your actual URL
pub const QWEN3_MAX: &str = "qwen3-max";

// Keep conversion traits consistent with other providers
impl_conversion_traits!(
    AsTranscription,
    AsImageGeneration,
    AsAudioGeneration for Client<T>
);
