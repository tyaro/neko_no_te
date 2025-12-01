# chat-history

会話履歴の永続化と管理を提供するクレートです。

## 機能

- 会話メッセージの永続化（JSON形式）
- 会話の作成・読み込み・削除
- 会話一覧の取得（メタデータベース）
- スレッドセーフな操作（`Arc<Mutex<>>`との組み合わせを想定）

## 使用例

### 基本的な使い方

```rust
use chat_history::{ConversationManager, Conversation, Message, MessageRole};

// マネージャーの初期化
let storage_dir = ConversationManager::default_storage_dir()?;
let manager = ConversationManager::new(storage_dir)?;

// 新しい会話を作成
let mut conversation = Conversation::new("My First Chat");
conversation.add_message(Message::new(
    MessageRole::User,
    "Hello, AI!".to_string(),
));

// 会話を保存
manager.save(&conversation)?;

// 会話を読み込み
let loaded = manager.load(&conversation.id)?;

// 会話一覧を取得
let all_conversations = manager.list_metadata()?;
```

### GUIアプリケーションとの統合

```rust
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

// スレッドセーフな会話管理
let conversation = Arc::new(Mutex::new(Conversation::new("Chat")));

// メッセージパッシング用チャネル
let (tx, rx) = mpsc::unbounded_channel();

// バックグラウンドでLLM呼び出し
let conversation_clone = conversation.clone();
tokio::spawn(async move {
    let response = call_llm("user input").await;
    let message = Message::new(MessageRole::Assistant, response);
    
    // 会話に追加
    conversation_clone.lock().unwrap().add_message(message.clone());
    
    // UIに通知
    tx.send(message).unwrap();
});

// UIスレッドで受信
while let Ok(message) = rx.try_recv() {
    // UI更新処理
}
```

## データ保存場所

デフォルトで `~/.neko-assistant/conversations/` に各会話がJSON形式で保存されます：

```
~/.neko-assistant/conversations/
  ├── <uuid-1>.json
  ├── <uuid-2>.json
  └── <uuid-3>.json
```

## Message型の構造

```rust
pub struct Message {
    pub id: String,            // UUID
    pub role: MessageRole,     // User/Assistant/System/Error
    pub content: String,       // メッセージ本文
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,  // 拡張用
}
```

## Conversation型の構造

```rust
pub struct Conversation {
    pub id: String,            // UUID
    pub title: String,         // 会話タイトル
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<Message>,
}
```

## テスト

```bash
cargo test -p chat-history
```

## 設計方針

- **シンプルなAPI**: 読み込み・保存・削除の3つの基本操作
- **JSON形式**: 人間が読める形式で保存（デバッグやバックアップが容易）
- **スレッドセーフ**: `Arc<Mutex<>>`と組み合わせて使用可能
- **拡張性**: `Message.metadata`で将来的な機能拡張に対応

## Phase 3での役割

Phase 3（会話履歴実装）では、以下の役割を果たします：

1. **永続化**: 会話を再起動後も保持
2. **状態管理の改善**: `Rc<RefCell<>>`から`Arc<Mutex<>>`への移行を可能に
3. **非同期対応**: バックグラウンドスレッドから安全に会話を更新
4. **会話の切り替え**: 複数の会話を管理し、UIで切り替え可能に

## ライセンス

MIT
