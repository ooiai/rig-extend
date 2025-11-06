# rig-extend

A collection of provider adapters that plug third‑party AI services into the Rig ecosystem. This repository is designed to grow over time: start with BaiLian (Alibaba DashScope), Volcengine (ByteDance Ark/Doubao), and TEI (Text Embedding Inference), and easily add more providers as your needs expand.

- rig-bailian: BaiLian (DashScope) integration for chat completions, embeddings, streaming, and reranking.
- rig-volcengine: Volcengine (Ark/Doubao) integration for chat completions, embeddings, and streaming.
- rig-tei: Local/remote TEI endpoints for embeddings and reranking, with simple endpoint overrides.

Each crate follows the same conventions so you can switch providers with minimal changes in your application code.

---

## Installation

Add one or more adapters to your Cargo.toml. Once published on crates.io, you can use:

```toml
[dependencies]
rig-bailian = "0.1"
rig-volcengine = "0.1"
rig-tei = "0.1"
rig = "0.23"           # Your Rig core version
rig-derive = "0.1"     # Optional: for derive macros like Embed
```

If you’re working directly from this repository (path dependencies):

```toml
[dependencies]
rig-bailian = { path = "rig-bailian" }
rig-volcengine = { path = "rig-volcengine" }
rig-tei = { path = "rig-tei" }
rig = "0.23"
rig-derive = "0.1"
```

---

## Quick start

Below are minimal examples for each provider. For more complete samples, see the examples directories linked at the end of this README.

### rig-bailian

Chat completion with an agent and embeddings using BaiLian (DashScope).

```rust
use rig::completion::Prompt;
use rig::prelude::*;
use rig_derive::Embed;

#[derive(rig_derive::Embed, Debug)]
struct Doc {
    #[embed]
    text: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure via environment (see variables below)
    let client = rig_bailian::Client::from_env();

    // Chat completion via agent
    let response = client
        .agent(rig_bailian::QWEN3_MAX)
        .context("You are a helpful assistant.")
        .prompt("Say hello in one sentence.")
        .await?;
    println!("BaiLian agent: {response}");

    // Embeddings
    let embeddings = client
        .embeddings(rig_bailian::TEXT_EMBEDDING_V4)
        .document(Doc { text: "Hello, world!".into() })?
        .document(Doc { text: "Goodbye, world!".into() })?
        .build()
        .await?;
    println!("BaiLian embeddings: {embeddings:?}");

    // Rerank (top-k = 2)
    let docs = vec![
        "Transformers are attention-based architectures.".to_string(),
        "Reranking orders documents by relevance.".to_string(),
    ];
    let rerank = client
        .rerank_model(rig_bailian::GTE_RERANK_V2, None)
        .rerank("what is a transformer?", &docs, Some(2), true)
        .await?;
    println!("BaiLian rerank: {rerank:?}");

    Ok(())
}
```

Environment variables:

- BAILIAN_API_KEY: Your DashScope API key.
- BAILIAN_BASE_URL: Optional. Defaults to https://dashscope.aliyuncs.com/compatible-mode/v1.

More examples: rig-bailian/examples

---

### rig-volcengine

Chat completion with an agent and embeddings using Volcengine (Ark/Doubao).

```rust
use rig::completion::Prompt;
use rig::prelude::*;
use rig_derive::Embed;

#[derive(rig_derive::Embed, Debug)]
struct Doc {
    #[embed]
    text: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure via environment (see variables below)
    let client = rig_volcengine::Client::from_env();

    // Chat completion via agent
    let response = client
        // Replace with your endpoint/model id
        .agent("ep-xxxxxxxxxxxxxx")
        .context("You are a helpful assistant.")
        .prompt("Say hello in one sentence.")
        .await?;
    println!("Volcengine agent: {response}");

    // Embeddings (choose a constant or string id)
    let embeddings = client
        .embeddings(rig_volcengine::TEXT_DOUBAO_EMBEDDING)
        .document(Doc { text: "Hello, world!".into() })?
        .document(Doc { text: "Goodbye, world!".into() })?
        .build()
        .await?;
    println!("Volcengine embeddings: {embeddings:?}");

    Ok(())
}
```

Environment variables:

- VOLCENGINE_API_KEY: Your Volcengine API key.
- VOLCENGINE_BASE_URL: Optional. Defaults to https://ark.cn-beijing.volces.com/api/v3.

More examples: rig-volcengine/examples

---

### rig-tei

Use a local or remote TEI router for embeddings and reranking. You can set a base URL or override endpoints individually.

```rust
use rig::prelude::*;
use rig_derive::Embed;

#[derive(rig_derive::Embed, Debug)]
struct Doc {
    #[embed]
    text: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // From environment (TEI_BASE_URL), or build manually:
    let client = rig_tei::Client::builder()
        .embed_endpoint("http://localhost:6280")  // POST /embed
        .rerank_endpoint("http://localhost:6290") // POST /rerank
        .build();

    // Embeddings
    let embeddings = client
        // TEI may not require a model id; pass a placeholder if your router ignores it
        .embeddings("")
        .document(Doc { text: "Hello, world!".into() })?
        .document(Doc { text: "Goodbye, world!".into() })?
        .build()
        .await?;
    println!("TEI embeddings: {embeddings:?}");

    // Rerank
    let docs = vec![
        "Transformers are attention-based architectures.".to_string(),
        "Reranking orders documents by relevance.".to_string(),
    ];
    let rerank = client.rerank("what is a transformer?", docs, Some(2)).await?;
    println!("TEI rerank: {rerank:?}");

    Ok(())
}
```

Environment variables:

- TEI_BASE_URL: Optional. Defaults to http://127.0.0.1:8080. You can also override `embed`/`rerank` endpoints via the builder.

More examples: rig-tei/examples

---

## Environment variables (summary)

- BaiLian (DashScope)
  - BAILIAN_API_KEY: Required.
  - BAILIAN_BASE_URL: Optional override. Default: https://dashscope.aliyuncs.com/compatible-mode/v1

- Volcengine (Ark/Doubao)
  - VOLCENGINE_API_KEY: Required.
  - VOLCENGINE_BASE_URL: Optional override. Default: https://ark.cn-beijing.volces.com/api/v3

- TEI
  - TEI_BASE_URL: Optional. Default: http://127.0.0.1:8080
  - For fine-grained control, use `Client::builder().embed_endpoint(...).rerank_endpoint(...).predict_endpoint(...)`.

---

## Why rig-extend?

- Unified abstractions: Build agents, completions, embeddings, and reranking via a consistent API across providers.
- Easy switching: Swap providers with minimal code changes.
- Streaming support: Provider adapters expose the same streaming semantics used in Rig core.
- Production-ready: Typed requests/responses, error handling, and composable builders.

---

## Examples

- BaiLian (DashScope): rig-bailian/examples
  - agent_wirh_bailian.rs
  - bailian_embeddings.rs
  - bailian_rereank.rs

- Volcengine (Ark/Doubao): rig-volcengine/examples
  - agent_wirh_volcengine.rs
  - volcengine_embeddings.rs

- TEI: rig-tei/examples
  - tei_embeddings.rs
  - tei_rerank.rs

Run any example from the crate directory:

```bash
# From inside a crate directory, e.g., rig-bailian
cargo run --example agent_wirh_bailian
```

Make sure the necessary environment variables are set for the chosen provider.

---

## Extending Rig with new providers

This repository is intended to grow. To add a new adapter:

1. Create a new crate (e.g., rig-<provider>) in this workspace.
2. Implement a `Client` with:
   - `ProviderClient` for `from_env()` and `from_val(...)`.
   - The relevant capability traits (e.g., `CompletionClient`, `EmbeddingsClient`, etc.).
   - `VerifyClient` for a fast health check (if applicable).
3. Expose a consistent surface:
   - `Client::builder()` with `base_url(...)`, `.with_client(...)`, etc.
   - Model builders like `.agent(...)`, `.embeddings(...)`, and optional constants for well-known model ids.
4. Add examples for common flows (agent/completion, embeddings, reranking, streaming).
5. Update this README to include description, environment variables, and example links.

Following these patterns keeps the API uniform and makes it easy to adopt new providers.

---

## Development

- Edition: 2024
- Build:
  - cargo build
  - cargo test
  - cargo run --example <name> (from the specific crate directory)

If publishing to crates.io, ensure each crate has complete metadata (description, license, documentation, keywords, categories) and a curated include list in its Cargo.toml.

---

## License

Each crate is intended to be dual-licensed under MIT or Apache-2.0. See the individual crate’s Cargo.toml for details.
