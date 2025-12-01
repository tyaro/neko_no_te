# GPUI 使い方メモ (ローカル zed-fork)

このドキュメントは、リポジトリ内のローカル `gpui` (zed-fork) を使って `neko-assistant` 側で UI を作るときによく使うパターンと注意点を短くまとめたものです。

基本的な考え方

- `Application` を作り `run` で開始する。コールバック内で `App`（`cx`）を受け取り、ウィンドウやエンティティを作成する。
- UI の単位は `Entity<T>`（view）。`T` は `Render` を実装した型。`Render::render` が要素ツリー（`impl IntoElement`）を返す。
- ルートのウィンドウコンテンツは `cx.new(...)` で `Entity` を作って返す。

主要 API（抜粋）

- Application
  - `Application::new()` / `Application::headless()`
  - `run(|cx: &mut App| { ... })` — アプリ起動。`cx` は `App`。

- Window
  - `WindowOptions`, `WindowBounds`, `Bounds::centered(...)` などでウィンドウ初期配置を決める
  - `cx.open_window(options, |window, cx| { ... })` — ウィンドウを開き、クロージャでそのウィンドウのルート `Entity` を返す（`cx.new` を使うのが普通）

- Entity / View / Render
  - ある型 `MyView` に対して `impl gpui::Render for MyView { fn render(&mut self, window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl IntoElement { ... } }`
  - `IntoElement` を返すことで、`String` や他の `Entity`、組み込みの要素に変換され描画される。
  - ルートは `cx.new(|cx| MyView { ... })` で作成し、`cx.open_window(..., |_, cx| cx.new(|_| ...))` のように返す。

- Context (`cx`)
  - `cx.new(...)` — 新しいエンティティを作る（`Entity<T>` を返す）
  - `cx.spawn(...)` / `spawn_in` — 非同期タスクのスケジュール
  - `cx.observe(...)` / `cx.subscribe(...)` などで他エンティティの変化を監視
  - `cx.notify()` — 自分が変化したことを通知
  - `cx.activate(bool)` — アプリのアクティベート（例: ウィンドウをフォアグラウンドにする）

よくある最小実装例

```rust
use gpui::{prelude::*, Application, App, Bounds, WindowOptions, WindowBounds, size, px};

struct PluginListRender { plugins: Vec<(String,String)> }

impl gpui::Render for PluginListRender {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        // とりあえずテキストを返すだけ（IntoElement を実装しているためそのまま使える）
        let mut s = String::new();
        for (name, desc) in &self.plugins {
            if !s.is_empty() { s.push('\n'); }
            s.push_str(&format!("{}: {}", name, desc));
        }
        s
    }
}

fn run_gui_example(items: Vec<(String,String)>) {
    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.), px(600.)), cx);
        cx.open_window(
            WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() },
            move |_window, cx| {
                cx.new(|_| PluginListRender { plugins: items.clone() })
            },
        ).unwrap();
        cx.activate(true);
    });
}
```

注意点 / トラブルシュート

- IntoElement の実体は色々（`String`, `Entity<V>` 等）なので、返す型を間違えるとコンパイルエラーになる（例: 直に `String` を返すのか `cx.new(...)` を返すのか）。
- 複数の `gpui` バージョンが依存グラフに混在すると型が互換でなくなりコンパイルが通らない（今回の workspace では `Cargo.toml` の `[patch.crates-io]` でローカルパスに揃える運用をしている）。
- より高レベルなビルダー（`gpui-component` 等）を使うと楽だが、ローカル `gpui` とバージョン差があると互換性の問題が出るため、同じソースツリー／同バージョンに揃える必要がある。

次の実装ステップ提案

- プラグイン一覧用の `Entity` を `List`（選択可能）と `Detail` パネルに分けて実装する。
- 有効化フロー: `Enable` ボタンで能力（capabilities）を列挙した確認ダイアログを表示し、確認後 `spawn_guarded` 相当の API を呼ぶ。
- 将来的に `gpui-component` を使う場合は、`zed-fork` 側に合わせてフォーク or バージョン固定するか、ローカル化してビルダー API を修正する。

参照

- リポジトリ内の `crates/gpui/src` を参照すると `Context`/`Element`/`Window` の細かい振る舞いを確認できます。

---
このファイルは短く要点をまとめたメモです。UI 実装を進める際に、具体的な要素（リスト・ボタン・スクロール等）の例が欲しければ続けて作成します。
