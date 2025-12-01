# model-adapter

モデル固有の呼び出しフォーマット（function-calling / tools / prompt 形式）を内包するアダプタ抽象と、デフォルトアダプタを提供します。

## 目的

コアは `ModelProvider` を通した汎用的な generate を担い、各モデル固有の入力整形や関数呼び出しフォーマットは `ModelAdapter` に委ねることで拡張性を確保します。

## 組み込みアダプタ

### Llama3DefaultAdapter
- サポート: `llama3.1:8b`
- `ToolSpec` を JSON にシリアライズしてプロンプト末尾に付与

### Phi4MiniAdapter
- サポート: `phi4-mini:3.8b`, `Phi-4-mini-instruct`
- Phi-4-mini-instruct の専用チャットフォーマット `<|system|>...<|end|>`
- ネイティブな関数呼び出しサポート (`<|tool|>...</|tool|>`)

## 使用例

```rust
use model_adapter::{Phi4MiniAdapter, ModelAdapter, ToolSpec};
use model_provider::OllamaProvider;

let adapter = Phi4MiniAdapter::new();
let provider = OllamaProvider::new("http://localhost:11434/")?;

// 通常の会話
let result = adapter.invoke(&provider, "phi4-mini:3.8b", "Hello!", None).await?;

// ツール付き会話
let tools = vec![
    ToolSpec::with_parameters(
        "get_weather",
        "Get weather for a location",
        serde_json::json!({
            "location": {"type": "str", "description": "City name"}
        })
    )
];
let result = adapter.invoke(&provider, "phi4-mini:3.8b", "What's the weather?", Some(&tools)).await?;
```

## 拡張方法

追加モデル対応は `crates/plugins/` または `plugins/` に新しいクレートを作成し、`ModelAdapter` を実装してください。
