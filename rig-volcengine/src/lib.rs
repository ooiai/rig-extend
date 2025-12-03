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
pub use embedding::{EmbeddingModel, TEXT_DOUBAO_EMBEDDING, TEXT_DOUBAO_EMBEDDING_LARGE};

// Constants (aligned with original single-file version)
pub const VOLCENGINE_API_BASE_URL: &str = "https://ark.cn-beijing.volces.com/api/v3";
pub const DOUBAO_SEED: &str = "Doubao-Seed-1.6";
