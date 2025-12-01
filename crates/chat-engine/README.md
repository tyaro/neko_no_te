# chat-engine

neko-assistant のチャットエンジンクレート。ModelProvider と ModelAdapter を使用して会話を管理します。

## 機能

- メッセージ履歴管理
- 非同期メッセージ送信
- 履歴の自動トリミング
- ツール/関数呼び出しサポート
- システムプロンプト設定

## 使用例

```rust
use chat_engine::{ChatEngine, Message, Role};
use model_adapter::Phi4MiniAdapter;
use model_provider::OllamaProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // プロバイダとアダプタの初期化
    let provider = Arc::new(OllamaProvider::new("http://localhost:11434/")?);
    let adapter = Arc::new(Phi4MiniAdapter::new());
    
    // チャットエンジンの作成
    let mut engine = ChatEngine::new(
        provider,
        adapter,
        "phi4-mini:3.8b".to_string(),
    )
    .with_system_prompt("You are a helpful assistant.".to_string())
    .with_max_history(100);
    
    // メッセージ送信
    let response = engine.send_message("Hello!").await?;
    println!("AI: {}", response);
    
    // 履歴の取得
    for msg in engine.get_history() {
        println!("{:?}: {}", msg.role, msg.content);
    }
    
    Ok(())
}
```

## ツール呼び出しの例

```rust
use model_adapter::ToolSpec;

let tools = vec![
    ToolSpec::with_parameters(
        "get_weather",
        "Get current weather",
        serde_json::json!({
            "location": {"type": "str", "description": "City name"}
        })
    )
];

let response = engine.send_message_with_tools(
    "What's the weather in Tokyo?",
    Some(&tools)
).await?;
```

## API

### ChatEngine

- `new()` - 新しいエンジンを作成
- `with_system_prompt()` - システムプロンプトを設定
- `with_max_history()` - 最大履歴数を設定
- `send_message()` - メッセージを送信
- `send_message_with_tools()` - ツール付きメッセージを送信
- `get_history()` - メッセージ履歴を取得
- `clear_history()` - 履歴をクリア

### Message

- `new()` - 新しいメッセージを作成
- `user()` - ユーザーメッセージを作成
- `assistant()` - アシスタントメッセージを作成
- `system()` - システムメッセージを作成
