# ollama-client

Lightweight Rust client for talking to a local Ollama HTTP endpoint.

Usage
- Create a client with the base URL of your local Ollama instance (example uses the common `http://localhost:11434/`):

```rust
use ollama_client::OllamaClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = OllamaClient::new("http://localhost:11434/")?;
    let healthy = client.health().await?;
    println!("health: {}", healthy);

    let resp = client.generate("llama2", "Say hello in Japanese.").await?;
    println!("response: {}", resp);
    Ok(())
}
```

Notes
- The crate posts JSON to `/api/generate` relative to the configured base URL. If your local installation uses a different endpoint, adjust the base URL accordingly.
- The crate returns the raw response body as a string. If you prefer structured responses, deserialize into a custom type using `serde_json`.
