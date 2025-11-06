# rig-volcengine

Rig adapter for Volcengine (ByteDance Ark/Doubao). This crate integrates Volcengine’s OpenAI‑compatible APIs into the Rig ecosystem with a consistent, strongly‑typed interface for:
- Chat completions (agents)
- Text embeddings
- Streaming responses

Use this adapter to swap Volcengine in and out with other providers supported by Rig with minimal code changes.

Documentation: https://docs.rs/rig-volcengine
Repository: https://github.com/ooiai/rig-extend


## Features

- Consistent Rig API surface:
  - `Client::from_env()` and `Client::builder(...)`
  - `.agent(model)`, `.embeddings(model)`
- Chat completion with context stacking (agents)
- Text embedding builders with `derive(Embed)`
- Streaming support aligned with Rig semantics
- Sensible defaults and environment‑based configuration

Key constants:
- `VOLCENGINE_API_BASE_URL`: default base URL (`https://ark.cn-beijing.volces.com/api/v3`)
- `TEXT_DOUBAO_EMBEDDING`, `TEXT_DOUBAO_EMBEDDING_LARGE`: convenience model ids for embeddings
- `DOUBAO_SEED`: a convenience seed value for Doubao (if needed by your flow)


## Installation

From crates.io (recommended):

```toml
[dependencies]
rig-volcengine = "0.1"
rig = "0.23"           # Rig core
rig-derive = "0.1"     # Optional: for derive macros like Embed
```

From a workspace/path (if you’re developing locally):

```toml
[dependencies]
rig-volcengine = { path = "../rig-volcengine" }
rig = "0.23"
rig-derive = "0.1"
```


## Configuration (Environment Variables)

- `VOLCENGINE_API_KEY` (required): Your Volcengine API key.
- `VOLCENGINE_BASE_URL` (optional): Override API base. Defaults to:
  `https://ark.cn-beijing.volces.com/api/v3`.

Example:

```bash
export VOLCENGINE_API_KEY="ak-xxxxxxxx"
# export VOLCENGINE_BASE_URL="https://ark.cn-beijing.volces.com/api/v3"
```


## Quick Start

Below are minimal snippets for chat and embeddings.

### Chat (Agent)

```rust
use rig::completion::Prompt;
use rig::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure via env: VOLCENGINE_API_KEY, optional VOLCENGINE_BASE_URL
    let client = rig_volcengine::Client::from_env();

    // Build an agent with context; replace ep-... with your endpoint/model id
    let response = client
        .agent("ep-xxxxxxxxxxxxxx")
        .context("You are a concise, helpful assistant.")
        .prompt("Say hello in one sentence.")
        .await?;

    println!("Volcengine agent: {response}");
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
    let client = rig_volcengine::Client::from_env();

    // Choose a model id (or use a literal, e.g., "doubao-embedding-text-240715")
    let embeddings = client
        .embeddings(rig_volcengine::TEXT_DOUBAO_EMBEDDING)
        .document(Doc { text: "Hello, world!".into() })?
        .document(Doc { text: "Goodbye, world!".into() })?
        .build()
        .await?;

    println!("{embeddings:?}");
    Ok(())
}
```


## Examples

More end‑to‑end samples are available in this crate’s examples directory:

- `agent_wirh_volcengine.rs`
- `volcengine_embeddings.rs`

Run from this crate directory:

```bash
# Make sure VOLCENGINE_API_KEY is set
cargo run --example agent_wirh_volcengine
cargo run --example volcengine_embeddings
```


## Versioning and Compatibility

- Rust edition: 2024
- This crate aligns its public surface with the Rig core abstractions. Check your `rig` crate version for compatibility (examples use `rig = "0.23"`).


## License

MIT. See the `LICENSE` file (or package metadata) for details.
