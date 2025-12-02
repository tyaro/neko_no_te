use ollama_client::OllamaClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = OllamaClient::new("http://localhost:11434/")?;
    println!("health: {:?}", client.health().await);

    let r = client
        .generate("llama2", "Translate to Japanese: 'The quick brown fox'")
        .await;
    println!("generate: {:?}", r);
    Ok(())
}
