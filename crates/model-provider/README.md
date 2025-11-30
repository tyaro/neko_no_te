# model-provider

Shared abstraction for model/LLM providers used in the project.

This crate exposes:
- `ModelProvider` trait — async trait for `name`, `health`, and `generate`.
- `GenerateResult` — textual + optional structured response container.
- `ProviderError` — common error type.

Adapters for concrete backends should implement `ModelProvider`. The
crate offers an optional `ollama-impl` feature which provides a thin adapter
over the local `ollama-client` crate.

Example (with feature enabled):

```rust
use model_provider::ollama_impl::OllamaProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let p = OllamaProvider::new("http://localhost:11434/")?;
    println!("health: {}", p.health().await?);
    let r = p.generate("llama2", "Hello").await?;
    println!("text: {}", r.text);
    Ok(())
}
```
