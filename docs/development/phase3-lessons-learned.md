# Phase 3: 会話履歴機能 - 振り返りと教訓

## 完成日
2025年12月2日

## 実装した機能
1. **chat-history クレート**: 会話の永続化とメタデータ管理
2. **会話一覧サイドバー**: 過去の会話を表示・切り替え
3. **自動保存機能**: メッセージ追加時に自動的にJSON保存
4. **非ブロッキングUI**: LLM呼び出しをバックグラウンドで実行
5. **メッセージハンドラー分離**: UI層とビジネスロジックの完全分離

## 遭遇した問題と解決策

### 1. 起動時フリーズ（Critical Bug）
**問題**: サイドバー実装後、アプリケーションが起動時にクラッシュ（exit code: 0xcfffffff）

**原因**:
- `render()` メソッド内で `lock().unwrap()` を使用
- 複数箇所で同じ Mutex にロックを取得しようとしてデッドロック/パニック
- ロック保持中に長い処理を行い、他の部分がブロック

**解決策**:
```rust
// ❌ 間違い
let conv = self.conversation.lock().unwrap();
// 長い処理...
// convを使い続ける

// ✅ 正解
let messages = if let Ok(conv) = self.conversation.try_lock() {
    conv.messages.clone()  // すぐにクローンして解放
} else {
    Vec::new()  // ロック失敗時のフォールバック
};
// ロックはすぐに解放される
```

**教訓**:
- **UI層では `lock().unwrap()` を使わない**
- **`try_lock()` を使い、失敗時のフォールバックを用意**
- **ロック保持時間を最小化**: データをクローンしてすぐ解放
- **render() は高頻度で呼ばれる**: 重い処理やブロッキング操作は避ける

### 2. 入力フィールドのラグ
**問題**: メッセージ送信後、入力フィールドが重くなる

**原因**:
- 入力イベントハンドラー内で `cx.notify()` を呼び出し
- UI更新チャネル経由で既に通知しているのに、さらに全体を再描画
- 二重の再描画でパフォーマンス低下

**解決策**:
```rust
// ❌ 間違い
let _ = ui_tx_sub.send(());
cx.notify();  // 不要な全体再描画

// ✅ 正解
let _ = ui_tx_sub.send(());
// cx.notify()はUI更新チャネル経由で行うため、ここでは呼ばない
```

**教訓**:
- **通知チャネルを使っている場合、`cx.notify()` は不要**
- **イベント駆動設計では、通知の重複を避ける**

### 3. スクロールが機能しない
**問題**: メッセージが増えてもスクロールできない

**原因 A（初期）**:
- `track_scroll()` の構造が不適切
- `overflow_hidden` と `track_scroll` のネストが間違っている

**解決策 A**:
```rust
// ✅ 正しいネスト構造
div()
    .flex_1()
    .overflow_hidden()  // 外側で overflow を制御
    .child(
        div()
            .size_full()
            .overflow_y_scroll()  // スクロール可能に
            .track_scroll(handle)  // スクロールトラッキング
            .child(/* コンテンツ */)
    )
```

**原因 B（再発）**:
- メッセージ領域に高さ制限がない
- `main_content` が `h_full()` を持っていない
- flexbox の高さ継承が機能していない

**解決策 B**:
```rust
// ❌ 間違い: 高さ制約なし
let main_content = div()
    .flex_1()
    .v_flex()
    .child(toolbar)
    .child(msgs_container)
    .child(input_area);

div().h_flex().size_full()

// ✅ 正解: 明示的な高さ制約
let main_content = div()
    .flex_1()
    .h_full()  // 高さを親から継承
    .v_flex()
    .child(toolbar)
    .child(msgs_container)
    .child(input_area);

div().h_flex().w_full().h_full()  // 明示的に指定
```

**教訓**:
- **スクロールコンテナには親の高さ制約が必須**
- **flexbox では `h_full()` と `flex_1()` を組み合わせる**
- **`size_full()` より `w_full().h_full()` の方が明確**

### 4. UI層にビジネスロジックが混在
**問題**: `ChatView` の `new()` メソッドが142行、LLM呼び出しロジックがUI層に直接埋め込まれている

**原因**:
- 責務の分離が不十分
- イベントハンドラー内で `tokio::spawn` を直接使用
- UIコンポーネントがLLMの詳細を知りすぎている

**解決策**:
```rust
// ❌ 間違い: UI層で直接spawn
cx.subscribe_in(&input_state, window, move |...| {
    // 100行のLLM呼び出しロジック
    tokio::spawn(async move {
        let engine = LangChainEngine::new(...);
        // ...
    });
});

// ✅ 正解: MessageHandlerに分離
// 1. message_handler.rs を作成
pub struct MessageHandler { /* ... */ }
impl MessageHandler {
    pub fn handle_user_message(&self, input: String) {
        // LLM呼び出しロジック
    }
}

// 2. ChatView はシンプルに
let handler = message_handler.clone();
cx.subscribe_in(&input_state, window, move |...| {
    handler.handle_user_message(user_input);
});
```

**教訓**:
- **UI層はイベント受信と描画のみに専念**
- **ビジネスロジックは別モジュールに分離**
- **イベントハンドラーは薄く保つ（10行以内が理想）**
- **`tokio::spawn` はUI層で直接使わない**

## アーキテクチャの改善

### Before（Phase 3 初期）
```
[ChatView]
  ├─ UI描画
  ├─ 入力イベントハンドラー（142行）
  │   ├─ ユーザーメッセージ追加
  │   ├─ tokio::spawn でLLM呼び出し
  │   ├─ 会話保存
  │   └─ UI更新通知
  └─ 会話状態管理
```

### After（Phase 3 完成）
```
[ChatView]
  ├─ UI描画
  └─ 入力イベントハンドラー（14行）
      └─ MessageHandler に委譲

[MessageHandler]
  ├─ ユーザーメッセージ追加
  ├─ LLM呼び出し（バックグラウンド）
  ├─ 会話保存
  └─ UI更新通知

[ConversationManager]
  └─ JSON永続化
```

## ベストプラクティス

### Mutex の使用
```rust
// UI層（render メソッド内）
✅ try_lock() + clone + すぐ解放
❌ lock().unwrap() + 長時間保持

// バックグラウンドスレッド
✅ lock().unwrap() でOK（UIをブロックしない）
```

### UI更新通知
```rust
✅ チャネル経由で通知 → render() で cx.notify()
❌ イベントハンドラーで直接 cx.notify()（二重通知）
```

### Flexbox とスクロール
```rust
// スクロールコンテナの黄金パターン
div()
    .flex_1()           // 親の残りスペースを占有
    .h_full()           // 高さを明示的に継承
    .overflow_hidden()  // 外側でoverflow制御
    .child(
        div()
            .size_full()
            .overflow_y_scroll()
            .track_scroll(handle)
            .child(/* コンテンツ */)
    )
```

### モジュール分割
```rust
// ファイル構成
neko-assistant/
  src/
    gui/
      chat.rs          ← UI のみ（描画・イベント受信）
    message_handler.rs ← ビジネスロジック（LLM・永続化）

// 責務
[UI層]        入力受付 → ハンドラー呼び出し → 描画
[Handler層]   処理実行 → 状態更新 → UI通知
[Storage層]   データ永続化
```

## 次フェーズへの示唆

### Phase 4 の機能候補
1. **新規会話作成**: "New Chat" ボタンの実装
2. **会話切り替え**: サイドバーのアイテムクリックで会話ロード
3. **会話削除**: コンテキストメニューで削除機能
4. **会話タイトル編集**: インライン編集または専用ダイアログ
5. **会話検索**: タイトルや内容での検索機能

### 設計上の注意点
- **会話切り替え時のロック**: 現在の会話を保存 → 新しい会話をロード
- **UI状態のリセット**: スクロール位置、入力フィールドをクリア
- **非同期ロード**: 大きな会話ファイルは非ブロッキングでロード
- **エラーハンドリング**: ファイル読み込み失敗時のフォールバック

### 技術的改善
- `message_handler` の未使用警告を解消（実際には使用中）
- `Arc<MessageHandler>` を `ChatView` のフィールドから削除して、イベントハンドラーのみで保持
- スクロールの自動/手動切り替えロジックの改善
- 会話リストの自動更新（新規作成・削除時）

## まとめ

Phase 3 では以下を学びました：

1. **Mutexのベストプラクティス**: `try_lock()` とクローンで安全性を確保
2. **UI層の責務**: 描画と入力受付のみ、ビジネスロジックは別モジュール
3. **イベント駆動設計**: 通知チャネルで疎結合を実現
4. **Flexboxとスクロール**: 高さ制約とネスト構造が重要
5. **デバッグ手法**: 段階的な修正とロック競合の追跡

これらの教訓を次のフェーズでも活かしていきます。
