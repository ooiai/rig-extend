# rig-tei

Rig adapter for TEI (Text Embedding Inference). This crate integrates local or remote TEI-style endpoints into the Rig ecosystem through a consistent, strongly-typed interface for:
- Text embeddings
- Reranking
- Simple endpoint overrides (per feature)

Documentation: https://docs.rs/rig-tei
Repository: https://github.com/ooiai/rig-extend


## Features

- Consistent Rig API surface:
  - `Client::from_env()` and `Client::builder()`
  - `.embeddings(model)` builder API
  - `rerank(query, docs, top_k)` convenience method
- Flexible routing:
  - Global `TEI_BASE_URL` or per-feature endpoint overrides (`embed_endpoint`, `rerank_endpoint`, `predict_endpoint`)
- Works with local or remote TEI routers
- Zero-auth by default (aligns with many local setups), but easily extensible if your router requires auth
- Typed requests/responses and structured errors aligned with Rig

Key defaults:
- `TEI_DEFAULT_BASE_URL`: `http://127.0.0.1:8080`


## Installation

From crates.io (recommended):

```toml
[dependencies]
rig-tei = "0.1"
rig = "0.23"           # Rig core
rig-derive = "0.1"     # Optional: for derive macros like Embed
```

From a workspace/path (if you’re developing locally):

```toml
[dependencies]
rig-tei = { path = "../rig-tei" }
rig = "0.23"
rig-derive = "0.1"
```


## Configuration (Environment Variables)

- `TEI_BASE_URL` (optional): Override the base URL for the TEI router. Defaults to:
  `http://127.0.0.1:8080`

Example:

```bash
export TEI_BASE_URL="http://localhost:8080"
```

You can also bypass the environment variable and override feature endpoints directly via the builder:
- `.embed_endpoint("http://localhost:6280")`
- `.rerank_endpoint("http://localhost:6290")`
- `.predict_endpoint("http://localhost:6300")` (if your router supports it)


## Quick Start

Below are minimal snippets for embeddings and reranking.

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
    // Option A) Use environment (TEI_BASE_URL) or default http://127.0.0.1:8080
    // let client = rig_tei::Client::from_env();

    // Option B) Override specific endpoints (handy for local dev)
    let client = rig_tei::Client::builder()
        .embed_endpoint("http://localhost:6280")  // POST /embed
        .build();

    // Some TEI routers do not require a model id; pass "" if unused
    let embeddings = client
        .embeddings("")
        .document(Doc { text: "Hello, world!".into() })?
        .document(Doc { text: "Goodbye, world!".into() })?
        .build()
        .await?;

    println!("{embeddings:?}");
    Ok(())
}
```

### Reranking

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Option A) Use environment (TEI_BASE_URL) or default http://127.0.0.1:8080
    // let client = rig_tei::Client::from_env();

    // Option B) Override rerank endpoint
    let client = rig_tei::Client::builder()
        .rerank_endpoint("http://localhost:6290") // POST /rerank
        .build();

    let docs = vec![
        "Transformers are attention-based architectures.".to_string(),
        "Reranking orders documents by relevance.".to_string(),
    ];

    // top_k = Some(2)
    let results = client
        .rerank("what is a transformer?", docs, Some(2))
        .await?;

    for r in results {
        println!("#{}/{} {:?}", r.index, r.relevance_score, r.text);
    }
    Ok(())
}
```


## Examples

More end‑to‑end samples are available in this crate’s examples directory:

- `tei_embeddings.rs`
- `tei_rerank.rs`

Run from this crate directory:

```bash
# If needed, export TEI_BASE_URL or override endpoints in code
cargo run --example tei_embeddings
cargo run --example tei_rerank
```


## Versioning and Compatibility

- Rust edition: 2024
- This crate aligns its public surface with the Rig core abstractions. Check your `rig` crate version for compatibility (examples use `rig = "0.23"`).


## License

MIT. See the `LICENSE` file (or package metadata) for details.
