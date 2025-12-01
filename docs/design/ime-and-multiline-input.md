# IME対応と複数行入力の実装ガイド

## 概要

GPUI 0.2.2での制限を踏まえた、段階的なIME対応と複数行入力の実装方針。

## 実装済み機能

### Phase 1: 基本構造 ✅
- `ScrollHandle` の追加によるスクロール機能
- チャットバブルスタイルのメッセージ表示
- 送信キー設定 (`send_key`) の追加
- 複数行入力エリアの基本レイアウト

## 次のステップ

### Phase 2: キーボード入力対応（簡易版）
現在の状態では、テキスト入力が機能していません。以下の実装が必要です：

1. **`EntityInputHandler` トレイトの実装**
   ```rust
   impl EntityInputHandler for ChatView {
       fn selected_text_range(&mut self, ...) -> Option<UTF16Selection> { ... }
       fn replace_text_in_range(&mut self, ...) { ... }
       fn replace_and_mark_text_in_range(&mut self, ...) { ... }
       // など
   }
   ```

2. **Window への InputHandler 登録**
   ```rust
   impl Render for ChatView {
       fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
           window.handle_input(&mut ElementInputHandler::new(cx.entity()));
           // ...
       }
   }
   ```

3. **キーイベントハンドリング**
   - `on_key_down` でキーイベントをキャプチャ
   - Enter / Ctrl+Enter でメッセージ送信
   - Shift+Enter / Enter で改行

### Phase 3: IME完全対応
Zedの実装を参考に：
- `marked_text_range` で変換中テキスト管理
- `bounds_for_range` でIME候補ウィンドウの位置決定
- プラットフォーム固有のIME処理（XIM, IMM32, NSTextInputClient）

## 既知の制限

- GPUI 0.2.2 の `Input` コンポーネントはIME対応が不完全
- カスタム実装が必要だが、低レベルAPI（`ElementInputHandler`）の理解が必須
- 完全な実装には時間がかかるため、段階的なアプローチを推奨

## 代替案

### 短期的な解決策
1. **クリップボード経由の入力**
   - 日本語テキストをクリップボードにコピー
   - アプリ内でペースト（Ctrl+V）
   - 現在は未実装だが、`replace_text_in_range` で実装可能

2. **外部エディタ連携**
   - テキストエディタで文章を作成
   - コピー&ペーストで入力
   - 長文入力には実用的

### 長期的な解決策
1. **GPUI のアップグレード**
   - 最新版（0.3.x+）でIME対応が改善されている可能性
   - breaking changes に注意が必要

2. **Zedの実装を移植**
   - `gpui/examples/input.rs` を参考に完全実装
   - `gpui/src/input.rs` の `ElementInputHandler` を活用

## 参考リソース

### Zedプロジェクトの関連ファイル
- `crates/gpui/src/input.rs` - InputHandler トレイト定義
- `crates/gpui/examples/input.rs` - 実装サンプル
- `crates/gpui/src/platform/linux/x11/xim_handler.rs` - XIM実装
- `crates/gpui/src/platform/mac/window.rs` - macOS IME実装
- `crates/gpui/src/platform/windows/events.rs` - Windows IME実装

### GPUIドキュメント
- `EntityInputHandler` トレイト
- `ElementInputHandler` 構造体
- `Window::handle_input()` メソッド
