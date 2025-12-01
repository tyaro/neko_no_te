# GPUI 非同期処理パターン調査結果

## 調査日時
2025年12月1日

## 調査目的
UIをブロックせずにバックグラウンドで非同期タスク（Ollama API呼び出し）を実行し、完了後にUIを更新する方法を確認する。

## 使用環境
- **GPUIバージョン**: 0.2.2 (`neko-assistant/Cargo.toml`より)
- **プロジェクト**: neko-assistant

---

## 調査結果サマリー

GPUI 0.2.2では以下の非同期処理APIが利用可能：

1. **`Context::spawn()`** - エンティティコンテキストから非同期タスクを起動
2. **`AsyncApp`** - `await`ポイントをまたいで保持できる非同期コンテキスト
3. **`background_executor()`** - バックグラウンドスレッドでの実行
4. **`foreground_executor()`** - メインスレッド（UIスレッド）での実行

---

## 主要APIの詳細

### 1. `Context::spawn()` メソッド

```rust
pub fn spawn<AsyncFn, R>(&self, f: AsyncFn) -> Task<R>
where
    T: 'static,
    AsyncFn: AsyncFnOnce(WeakEntity<T>, &mut AsyncApp) -> R + 'static,
    R: 'static,
```

**特徴**:
- エンティティの`Context`から呼び出す
- `WeakEntity<T>`を受け取るクロージャを渡す（メモリリーク防止）
- `AsyncApp`を使ってawaitポイントをまたいでエンティティを更新可能
- 返り値の`Task<R>`は保持するか`.detach()`で切り離す必要がある

### 2. `AsyncApp` コンテキスト

```rust
pub struct AsyncApp {
    pub(crate) app: Weak<AppCell>,
    pub(crate) background_executor: BackgroundExecutor,
    pub(crate) foreground_executor: ForegroundExecutor,
}
```

**主要メソッド**:
- `update<R>(&self, f: impl FnOnce(&mut App) -> R) -> Result<R>` - アプリ状態の更新
- `background_executor()` - バックグラウンド実行用Executor取得
- `foreground_executor()` - フォアグラウンド実行用Executor取得
- `spawn()` - フォアグラウンドで新しいタスクを起動

### 3. Executorの使い分け

#### BackgroundExecutor
```rust
impl BackgroundExecutor {
    pub fn spawn<R>(&self, future: impl Future<Output = R> + Send + 'static) -> Task<R>
    where R: Send + 'static;
}
```
- CPUバウンドな処理やブロッキングI/Oに使用
- スレッドプールで実行される
- `Send`制約が必要

#### ForegroundExecutor
```rust
// メインスレッド（UIスレッド）で実行
// `Send`制約不要
```
- UI更新など、メインスレッドで実行する必要がある処理に使用
- `!Send`型も扱える

---

## 実装パターン

### パターン1: Context::spawn()を使った基本パターン

```rust
use gpui::*;

struct ChatView {
    messages: Rc<RefCell<Vec<(MessageType, String)>>>,
    // ...
}

impl ChatView {
    fn send_message(&mut self, user_input: String, window: &mut Window, cx: &mut Context<Self>) {
        // ユーザーメッセージを即座に追加
        self.messages.borrow_mut().push((MessageType::User, user_input.clone()));
        cx.notify();

        // 処理中メッセージを表示
        self.messages.borrow_mut().push((MessageType::System, "Thinking...".to_string()));
        cx.notify();

        // 非同期タスクを起動（UIをブロックしない）
        let messages = self.messages.clone();
        let task = cx.spawn(async move |this, mut cx| {
            // バックグラウンドでOllama API呼び出し
            let result = {
                let engine = create_engine(); // 実際の実装に置き換え
                engine.send_message_simple(&user_input).await
            };

            // UI更新（メインスレッドで実行）
            let _ = cx.update(|cx| {
                this.update(cx, |view, cx| {
                    // 処理中メッセージを削除
                    let mut msgs = messages.borrow_mut();
                    if let Some(last) = msgs.last() {
                        if matches!(last.0, MessageType::System) && last.1.contains("Thinking") {
                            msgs.pop();
                        }
                    }

                    // 結果を追加
                    match result {
                        Ok(text) => msgs.push((MessageType::Assistant, text)),
                        Err(e) => msgs.push((MessageType::Error, format!("Error: {}", e))),
                    }

                    cx.notify(); // UIを再描画
                })
            });
        });

        // タスクを切り離して実行継続
        task.detach();
    }
}
```

### パターン2: background_executor()を明示的に使用

```rust
fn send_message(&mut self, user_input: String, window: &mut Window, cx: &mut Context<Self>) {
    self.messages.borrow_mut().push((MessageType::User, user_input.clone()));
    cx.notify();

    let messages = self.messages.clone();
    
    // バックグラウンド実行を明示
    let task = cx.spawn(async move |this, mut cx| {
        // バックグラウンドExecutorで重い処理を実行
        let bg_executor = cx.background_executor().clone();
        let result = bg_executor.spawn(async move {
            let engine = create_engine();
            engine.send_message_simple(&user_input).await
        }).await;

        // UI更新はフォアグラウンド（自動的にspawnのコンテキストで実行される）
        let _ = cx.update(|cx| {
            this.update(cx, |view, cx| {
                // UI更新処理
                // ...
                cx.notify();
            })
        });
    });

    task.detach();
}
```

### パターン3: spawn_in()を使ったウィンドウコンテキスト対応

```rust
fn send_message(&mut self, user_input: String, window: &mut Window, cx: &mut Context<Self>) {
    self.messages.borrow_mut().push((MessageType::User, user_input.clone()));
    cx.notify();

    let messages = self.messages.clone();
    
    // ウィンドウコンテキストを使ったspawn
    let task = cx.spawn_in(window, async move |this, mut cx| {
        let result = {
            let engine = create_engine();
            engine.send_message_simple(&user_input).await
        };

        // AsyncWindowContextを使ってUI更新
        let _ = cx.update(|_, window, cx| {
            this.update(cx, |view, cx| {
                let mut msgs = messages.borrow_mut();
                match result {
                    Ok(text) => msgs.push((MessageType::Assistant, text)),
                    Err(e) => msgs.push((MessageType::Error, format!("Error: {}", e))),
                }
                cx.notify();
            }).ok();
        });
    });

    task.detach();
}
```

---

## 重要な注意事項

### 1. WeakEntityの使用
`spawn()`に渡されるクロージャには`WeakEntity<T>`が渡されます。これは長時間実行されるタスク中にエンティティが破棄される可能性があるためです。

### 2. Task の管理
- `Task<R>`を返す関数は、必ず`.detach()`で切り離すか、変数に保持する必要があります
- 保持しない場合、タスクは即座にキャンセルされます

```rust
// ❌ NG: タスクが即座にキャンセルされる
cx.spawn(async move |_, _| { /* ... */ });

// ✅ OK: detachで切り離し
cx.spawn(async move |_, _| { /* ... */ }).detach();

// ✅ OK: フィールドに保持
self.pending_task = Some(cx.spawn(async move |_, _| { /* ... */ }));
```

### 3. cx.notify()の必要性
UI更新後は必ず`cx.notify()`を呼び出してGPUIに変更を通知し、再描画をトリガーする必要があります。

### 4. エラーハンドリング
`AsyncApp::update()`は`Result<T>`を返すため、適切なエラーハンドリングが必要です：

```rust
let _ = cx.update(|cx| {
    // updateが失敗する可能性がある（エンティティが破棄された等）
    this.update(cx, |view, cx| {
        // UI更新
    })
}).ok(); // または .unwrap_or_default() など
```

---

## 現在のコードとの比較

### 現在の問題コード（UIブロッキング）
```rust
// std::thread::spawn + tokio::runtime::block_on でUIをブロック
let handle = std::thread::spawn(move || {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        engine.send_message_simple(&user_input).await
    })
});
let result = handle.join().unwrap();
```

### 推奨パターン（UIノンブロッキング）
```rust
// Context::spawn でUIをブロックしない
let task = cx.spawn(async move |this, mut cx| {
    let result = engine.send_message_simple(&user_input).await;
    let _ = cx.update(|cx| {
        this.update(cx, |view, cx| {
            // UI更新
            view.messages.borrow_mut().push(...);
            cx.notify();
        })
    });
});
task.detach();
```

---

## Tokioランタイムとの統合

GPUI 0.2.2では、すでに内部でasync runtimeが動作しているため、追加のTokioランタイムは不要です。ただし、既存のTokio依存コード（`langchain-bridge`など）を使う場合は、`background_executor()`経由で実行することで統合できます。

### 参考: gpui_tokio クレート
Zedリポジトリには`gpui_tokio`クレートがあり、GPUIとTokioを統合する例が示されています：

```rust
// gpui_tokio::Tokio::spawn() の実装例
cx.read_global(|tokio: &GlobalTokio, cx| {
    let join_handle = tokio.runtime.spawn(f);
    cx.background_spawn(async move {
        join_handle.await
    })
})
```

本プロジェクトで同様の統合が必要な場合、このパターンを参考にできます。

---

## 次のステップ

1. `neko-assistant/src/gui/chat.rs`の`send_message`ロジックを`Context::spawn()`パターンに書き換え
2. `langchain-bridge`または`chat-engine`を非同期コンテキストから安全に呼び出せるようラッパーを作成
3. エラーハンドリングとユーザーフィードバック（ローディング表示等）の実装
4. タスクのキャンセル機能（ユーザーが途中で停止できる）の検討

---

## 参考リンク

- [GPUI Context API - spawn()](https://github.com/zed-industries/zed/blob/main/crates/gpui/src/app/context.rs)
- [GPUI AsyncApp](https://github.com/zed-industries/zed/blob/main/crates/gpui/src/app/async_context.rs)
- [GPUI Executor](https://github.com/zed-industries/zed/blob/main/crates/gpui/src/executor.rs)
- [GPUI Contexts Documentation](https://github.com/zed-industries/zed/blob/main/crates/gpui/docs/contexts.md)
