# langchain-bridge

LangChain-rust を neko-assistant に統合するためのブリッジクレート。

## 目的

既存の `chat-engine` と並行して LangChain ベースの実装を提供し、段階的に移行できるようにします。

## 機能

- ✅ Ollama 統合
- ✅ ConversationalChain による会話履歴管理
- 🔄 ストリーミング応答（実装中）
- 🔄 カスタムプロンプトテンプレート（予定）

## 使用例

```rust
use langchain_bridge::LangChainEngine;

let mut engine = LangChainEngine::new("http://localhost:11434", "phi4-mini:3.8b");
let response = engine.send_message("こんにちは").await?;
println!("Response: {}", response);
```

## 参照

- [langchain-rust 検証結果](../../research/langchain-rust-test/README.md)
- [評価ドキュメント](../../docs/research/langchain-rust-evaluation.md)
