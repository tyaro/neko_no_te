# model-provider

プロジェクトで利用するモデル（LLM）プロバイダ向けの共通抽象を提供するクレートです。

本クレートが提供する主な要素:
- `ModelProvider` トレイト — 非同期の `name`, `health`, `generate` を定義します。
- `GenerateResult` — テキスト応答と、必要に応じた構造化応答を格納する型。
- `ProviderError` — 共通エラー型。

具体的なバックエンド（Ollama、OpenAI、Copilot など）は `ModelProvider` を実装します。
オプション機能 `ollama-impl` を有効にすると、ローカル `ollama-client` を利用する薄いアダプタが利用できます。

例（`ollama-impl` 有効時）:

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
