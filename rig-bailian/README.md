# rig-bailian

Rig adapter for Alibaba BaiLian (DashScope). This crate integrates BaiLian’s OpenAI‑compatible APIs into the Rig ecosystem with a consistent, strongly‑typed interface for:
- Chat completions (agents)
- Text embeddings
- Streaming responses
- Reranking

Use this adapter to swap BaiLian in and out with other providers supported by Rig with minimal code changes.

Documentation: https://docs.rs/rig-bailian


## Features

- Consistent Rig API surface:
  - `Client::from_env()` and `Client::builder(...)`
  - `.agent(model)`, `.embeddings(model)`, and rerank model helpers
- Chat completion with context stacking (agents)
- Text embedding builders with `derive(Embed)`
- Streaming support aligned with Rig semantics
- Rerank API with top‑k selection
- Sensible defaults and environment‑based configuration

Key constants:
- `QWEN3_MAX`: a convenience model id for chat completions
- `TEXT_EMBEDDING_V4`: a convenience model id for embeddings
- `GTE_RERANK_V2`: a convenience rerank model id
- `BAILIAN_API_BASE_URL`: default base URL (`https://dashscope.aliyuncs.com/compatible-mode/v1`)


## Installation

From crates.io (recommended):

```toml
[dependencies]
rig-bailian = "0.1"
rig = "0.23"           # Rig core
rig-derive = "0.1"     # Optional: for derive macros like Embed
```

From a workspace/path (if you’re developing locally):

```toml
[dependencies]
rig-bailian = { path = "../rig-bailian" }
rig = "0.23"
rig-derive = "0.1"
```


## Configuration (Environment Variables)

- `BAILIAN_API_KEY` (required): Your DashScope API key.
- `BAILIAN_BASE_URL` (optional): Override API base. Defaults to:
  `https://dashscope.aliyuncs.com/compatible-mode/v1`.

Example:

```bash
export BAILIAN_API_KEY="sk-xxxxxxxx"
# export BAILIAN_BASE_URL="https://dashscope.aliyuncs.com/compatible-mode/v1"
```


## Quick Start

Below are minimal snippets for chat, embeddings, and reranking.

### Chat (Agent)

```rust
use rig::completion::Prompt;
use rig::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure via env: BAILIAN_API_KEY, optional BAILIAN_BASE_URL
    let client = rig_bailian::Client::from_env();

    // Build an agent with context
    let response = client
        .agent(rig_bailian::QWEN3_MAX)
        .context("You are a concise, helpful assistant.")
        .prompt("Say hello in one sentence.")
        .await?;

    println!("BaiLian agent: {response}");
    Ok(())
}
```

### Embeddings

```rust
use rig::Embed;
use rig::prelude::*;
use rig_derive::Embed;

#[derive(Embed, Debug)]
struct Doc {
    #[embed]
    text: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = rig_bailian::Client::from_env();

    let embeddings = client
        .embeddings(rig_bailian::TEXT_EMBEDDING_V4)
        .document(Doc { text: "Hello, world!".into() })?
        .document(Doc { text: "Goodbye, world!".into() })?
        .build()
        .await?;

    println!("{embeddings:?}");
    Ok(())
}
```

### Rerank

```rust
use rig::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = rig_bailian::Client::from_env();

    let docs = vec![
        "Transformers are attention-based architectures.".to_string(),
        "Reranking orders documents by relevance.".to_string(),
    ];

    let model = client.rerank_model(rig_bailian::GTE_RERANK_V2, None);
    let results = model
        .rerank("what is a transformer?", &docs, Some(2), true)
        .await?;

    for r in results {
        println!("#{}/{} => {}", r.index, r.relevance_score, r.text);
    }
    Ok(())
}
```


## Examples

More end‑to‑end samples are available in the examples directory of this crate:

- `agent_wirh_bailian.rs`
- `bailian_embeddings.rs`
- `bailian_rereank.rs`

Run from this crate directory:

```bash
# Make sure BAILIAN_API_KEY is set
cargo run --example agent_wirh_bailian
cargo run --example bailian_embeddings
cargo run --example bailian_rereank
```


## Versioning and Compatibility

- Rust edition: 2024
- This crate aligns its public surface with the Rig core abstractions. Check your `rig` crate version for compatibility (examples use `rig = "0.23"`).


## License

MIT. See the `LICENSE` file (or package metadata) for details.
