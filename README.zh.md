# rig-extend（中文说明）

本仓库收录了一组面向 Rig 生态的「提供商适配器」，用于将第三方 AI 服务无缝接入统一的 Rig 抽象。当前包含以下适配器并可持续扩展：
- rig-bailian：阿里巴巴百炼（DashScope）适配器，支持聊天补全（Agent）、文本向量、流式输出与重排。
- rig-volcengine：字节火山引擎（Ark/Doubao）适配器，支持聊天补全（Agent）、文本向量与流式输出。
- rig-tei：TEI（Text Embedding Inference）本地/远程路由适配器，支持文本向量与重排，提供端点覆写能力。

所有适配器遵循一致的 API 规范，便于你在应用中以最小改动切换不同提供商。

---

## 安装

在 `Cargo.toml` 中添加需要的适配器依赖，并将 Rig 核心版本更新为 `rig-core = "0.25.0"`。

从 crates.io 安装（推荐）：
```
[dependencies]
rig-bailian    = "0.1"
rig-volcengine = "0.1"
rig-tei        = "0.1"
rig-core       = "0.25.0"    # Rig 核心
rig-derive     = "0.1.9"     # 可选：派生宏（如 Embed）
```

从本仓库工作区（path 依赖）安装（用于本地开发）：
```
[dependencies]
rig-bailian    = { path = "rig-bailian" }
rig-volcengine = { path = "rig-volcengine" }
rig-tei        = { path = "rig-tei" }
rig-core       = "0.25.0"
rig-derive     = "0.1.9"
```

---

## 快速上手

以下示例展示了各适配器的最简用法。更完整的演示可参考各 crate 的 `examples` 目录。

### rig-bailian（百炼/DashScope）

聊天补全（Agent）+ 文本向量 + 重排：

```
use rig::completion::Prompt;
use rig::prelude::*;
use rig_derive::Embed;

#[derive(Embed, Debug)]
struct Doc {
    #[embed]
    text: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 从环境变量读取（见下文环境变量）
    let client = rig_bailian::Client::from_env();

    // 聊天补全（Agent）
    let response = client
        .agent(rig_bailian::QWEN3_MAX)
        .context("你是一个简洁且乐于助人的助手。")
        .prompt("请用一句话打个招呼。")
        .await?;
    println!("BaiLian agent: {response}");

    // 文本向量
    let embeddings = client
        .embeddings(rig_bailian::TEXT_EMBEDDING_V4)
        .document(Doc { text: "你好，世界！".into() })?
        .document(Doc { text: "再见，世界！".into() })?
        .build()
        .await?;
    println!("BaiLian embeddings: {embeddings:?}");

    // 文本重排（top-k = 2）
    let docs = vec![
        "Transformers 是基于注意力机制的架构。".to_string(),
        "Reranking 按相关性为文档排序。".to_string(),
    ];
    let results = client
        .rerank_model(rig_bailian::GTE_RERANK_V2, None)
        .rerank("什么是 transformer？", &docs, Some(2), true)
        .await?;
    println!("BaiLian rerank: {results:?}");

    Ok(())
}
```

环境变量：
- BAILIAN_API_KEY（必需）：DashScope API Key
- BAILIAN_BASE_URL（可选）：默认 `https://dashscope.aliyuncs.com/compatible-mode/v1`

更多示例：`rig-bailian/examples`

---

### rig-volcengine（火山引擎 Ark/Doubao）

聊天补全（Agent）+ 文本向量：

```
use rig::completion::Prompt;
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

    // 聊天补全（Agent），将 ep-xxxxxxxxxxxxxx 替换为你自己的端点/模型 ID
    let response = client
        .agent("ep-xxxxxxxxxxxxxx")
        .context("你是一个简洁且乐于助人的助手。")
        .prompt("请用一句话打个招呼。")
        .await?;
    println!("Volcengine agent: {response}");

    // 文本向量（使用常量或直接字符串模型 ID）
    let embeddings = client
        .embeddings(rig_volcengine::TEXT_DOUBAO_EMBEDDING)
        .document(Doc { text: "你好，世界！".into() })?
        .document(Doc { text: "再见，世界！".into() })?
        .build()
        .await?;
    println!("Volcengine embeddings: {embeddings:?}");

    Ok(())
}
```

环境变量：
- VOLCENGINE_API_KEY（必需）：Volcengine API Key
- VOLCENGINE_BASE_URL（可选）：默认 `https://ark.cn-beijing.volces.com/api/v3`

更多示例：`rig-volcengine/examples`

---

### rig-tei（本地/远程 TEI 路由）

文本向量 + 重排，支持基础 URL 或端点级别覆写：

```
use rig::prelude::*;
use rig_derive::Embed;

#[derive(Embed, Debug)]
struct Doc {
    #[embed]
    text: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Option A：使用 TEI_BASE_URL（默认 http://127.0.0.1:8080）
    // let client = rig_tei::Client::from_env();

    // Option B：手动覆写各功能端点（用于本地开发）
    let client = rig_tei::Client::builder()
        .embed_endpoint("http://localhost:6280")  // POST /embed
        .rerank_endpoint("http://localhost:6290") // POST /rerank
        .build();

    // 某些 TEI 路由不需要模型 ID，可传入空字符串占位
    let embeddings = client
        .embeddings("")
        .document(Doc { text: "你好，世界！".into() })?
        .document(Doc { text: "再见，世界！".into() })?
        .build()
        .await?;
    println!("TEI embeddings: {embeddings:?}");

    // 文本重排
    let docs = vec![
        "Transformers 是基于注意力机制的架构。".to_string(),
        "Reranking 按相关性为文档排序。".to_string(),
    ];
    let results = client
        .rerank("什么是 transformer？", docs, Some(2))
        .await?;
    println!("TEI rerank: {results:?}");

    Ok(())
}
```

环境变量：
- TEI_BASE_URL（可选）：默认 `http://127.0.0.1:8080`
- 亦可通过 Builder 覆写具体端点：`.embed_endpoint(..) .rerank_endpoint(..) .predict_endpoint(..)`

更多示例：`rig-tei/examples`

---

## 环境变量汇总

- BaiLian（DashScope）
  - BAILIAN_API_KEY：必需
  - BAILIAN_BASE_URL：可选，默认 `https://dashscope.aliyuncs.com/compatible-mode/v1`

- Volcengine（Ark/Doubao）
  - VOLCENGINE_API_KEY：必需
  - VOLCENGINE_BASE_URL：可选，默认 `https://ark.cn-beijing.volces.com/api/v3`

- TEI
  - TEI_BASE_URL：可选，默认 `http://127.0.0.1:8080`
  - 高级用法：通过 `Client::builder()` 覆写指定功能端点

---

## 运行示例

在对应 crate 目录下运行示例（以 rig-bailian 为例）：
```
# 请先确保相关环境变量已配置
cargo run --example agent_wirh_bailian
```

其他示例位置：
- `rig-bailian/examples`：`agent_wirh_bailian.rs`、`bailian_embeddings.rs`、`bailian_rereank.rs`
- `rig-volcengine/examples`：`agent_wirh_volcengine.rs`、`volcengine_embeddings.rs`
- `rig-tei/examples`：`tei_embeddings.rs`、`tei_rerank.rs`

---

## 版本与兼容性

- Rust edition：2024
- Rig 核心版本：建议使用 `rig-core = "0.25.0"`
- `rig-derive`（可选）：`0.1.9`

各适配器的公共接口与 Rig 核心抽象保持一致，升级时请留意 trait 签名变动并确保依赖版本一致。

---

## 为什么选择 rig-extend？

- 统一抽象：在不同提供商之间共享一致的 API（Agent、Completion、Embeddings、Rerank、Streaming）。
- 易于切换：几乎不改动应用代码即可替换提供商。
- 流式支持：适配器暴露与 Rig 核心一致的流式语义。
- 生产可用：类型化的请求/响应、完善的错误处理与可组合的 Builder。

---

## 扩展新的提供商

本仓库鼓励扩展，添加新的适配器大致步骤：
1. 在 workspace 新建 `rig-<provider>` crate。
2. 实现 `Client` 并对接：
   - `ProviderClient`：提供 `from_env()` 与 `from_val(...)`。
   - 能力 trait：如 `CompletionClient`、`EmbeddingsClient` 等。
   - `VerifyClient`：提供简单健康检查（如 `GET /models`），便于快速诊断。
3. 暴露一致的外部接口：
   - `Client::builder()`（含 `base_url(...)`、`.with_client(...)` 等）。
   - 模型构造器：`.agent(...)`、`.embeddings(...)`，可以提供常见模型 ID 常量。
4. 在 `examples/` 增加常用流程演示（completion、embeddings、rerank、streaming）。
5. 更新根 README 与中文 README，补充环境变量与示例说明。

---

## 许可证

本仓库根目录下的许可证为 Apache-2.0（见 `LICENSE`）。各子 crate 的具体许可证声明请参考各自的 `Cargo.toml` 与随包元数据。
