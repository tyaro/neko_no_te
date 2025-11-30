# ollama-client

ローカルに起動した Ollama HTTP エンドポイントとやり取りするための軽量な Rust クライアントです。

## 使用例

ローカルの Ollama のベース URL（例: `http://localhost:11434/`）を指定してクライアントを作成します。

```rust
use ollama_client::OllamaClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = OllamaClient::new("http://localhost:11434/")?;
    let healthy = client.health().await?;
    println!("health: {}", healthy);

    let resp = client.generate("llama2", "日本語で挨拶してください。").await?;
    println!("response: {}", resp);
    Ok(())
}
```

## 注意点

- このクレートは設定されたベース URL に対して相対パス `/api/generate` に JSON を POST します。
 ローカル環境で異なるエンドポイントを使っている場合はベース URL を調整してください。
- レスポンスは生の本文（文字列）で返します。構造化された応答が必要な場合は `serde_json` でデシリアライズしてください。
