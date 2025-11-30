# GPUI Examples

このファイルは `crates/gpui` を使った簡単なコード例を集めたものです。実際の API 名はプロジェクト内の `crates/gpui` を参照してください。

- 1 最小ウィンドウ

```rust
// 疑似コード: メインウィンドウとイベントループ
fn main() {
    let app = gpui::App::new();
    let mut win = gpui::Window::new("ネコの手 - サンプル");
    win.set_min_size(800, 600);
    app.run(move |ctx| {
        ctx.clear();
        ctx.label("Hello GPUI");
    });
}
```

- 2 メニューとツールバー

```rust
// 疑似コード: メニュー定義
app.menu(|m| {
    m.menu("File", |f| {
        f.item("Open", || { /* open */ });
        f.item("Quit", || { /* quit */ });
    });
    m.menu("Help", |h| { h.item("About", || { /* about */ }); });
});

// ツールバー
toolbar.button("Run", || { /* run action */ });
```

- 3 リスト（プラグイン一覧）と選択

```rust
// 疑似コード: プラグインメタを読み込み、リストで表示
let plugins = load_plugins_from_disk("./plugins");
ui.list(&plugins, |ui, plugin| {
    ui.row(|r| {
        r.checkbox(&mut plugin.enabled);
        r.label(&plugin.name);
        r.label(&plugin.version);
    });
});
```

- 4 モーダルダイアログ（確認）

```rust
if ui.button("Delete") {
    ui.open_modal("confirm_delete");
}

ui.modal("confirm_delete", || {
    ui.label("本当に削除しますか?");
    if ui.button("はい") { perform_delete(); ui.close_modal(); }
    if ui.button("いいえ") { ui.close_modal(); }
});
```

- 5 非同期処理と UI 更新

```rust
// 長時間処理は別スレッドへ移し、結果をチャネルで受け取る
std::thread::spawn(move || {
    let result = heavy_work();
    tx.send(result).unwrap();
});

// UI 側でチャネルをポーリングして再描画
if let Ok(res) = rx.try_recv() {
    ui.label(&format!("完了: {}", res));
}
```

補足: 上のコードは概念を示す擬似コードです。実際の型やメソッド名は `crates/gpui` の API を参照して置き換えてください。

***
より詳細な実装例が必要なら、`neko-assistant` の GUI 統合例を作成してコミットします。希望する例（プラグイン管理画面、会話リスト、エディタ統合など）を教えてください。
