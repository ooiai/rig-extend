use rig::prelude::*;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create Bailian client and model
    let client = rig_bailian::Client::from_env();
    // Choose model constant
    let model = client.rerank_model(rig_bailian::GTE_RERANK_V2, None);

    let docs = vec![
        "Transformers are attention-based architectures.".to_string(),
        "Reranking orders documents by relevance.".to_string(),
    ];
    let results = model
        .rerank("what is a transformer?", &docs, Some(2), true)
        .await?;
    for r in results {
        println!("#{}/{} => {}", r.index, r.relevance_score, r.text);
    }
    Ok(())
}
