use rig::Embed;
use rig::prelude::*;
use rig_derive::Embed;

#[derive(Embed, Debug)]
struct Greetings {
    #[embed]
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create Tei client and model
    // let client = rig_tei::Client::from_env();
    let client = rig_tei::Client::builder()
        .embed_endpoint("http://localhost:6280")
        .build();

    // Prompt the model and print its response
    let embeddings = client
        .embeddings("")
        .document(Greetings {
            message: "Hello, world!".to_string(),
        })?
        .document(Greetings {
            message: "Goodbye, world!".to_string(),
        })?
        .build()
        .await
        .expect("Failed to embed documents");

    println!("{embeddings:?}");
    Ok(())
}
