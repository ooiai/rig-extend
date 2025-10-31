#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create Tei client and model
    // let client = rig_tei::Client::from_env();
    let client = rig_tei::Client::builder()
        .rerank_endpoint("http://localhost:6290")
        .build();

    let docs = vec![
        "Transformers are attention-based architectures.".to_string(),
        "Reranking orders documents by relevance.".to_string(),
    ];
    let results = client
        .rerank("what is a transformer?", docs, Some(2))
        .await?;
    for r in results {
        println!("#{}/{} {:?}", r.index, r.relevance_score, r.text);
    }
    Ok(())
}
