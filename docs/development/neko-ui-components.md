# neko-ui クレートの作成

## 概要

カスタムUIコンポーネントを`neko-ui`クレートに切り出しました。これにより、再利用可能なコンポーネントライブラリとして管理できるようになりました。

## 作成したコンポーネント

> **2025-12-04 更新**: TextInput コンポーネントは Phase 5 で `gpui-component::InputState`
> ベースの `chat_input` に集約したため現在は削除済み。以下は現行で維持している
> コンポーネントの一覧。

### 1. ChatBubble

チャットメッセージ表示用のバブルコンポーネント

**特徴**:
- 4種類のメッセージタイプ（User, Assistant, System, Error）
- 自動カラーリング
- ビルダーパターン対応

**使用例**:
```rust
use neko_ui::{ChatBubble, MessageType};

let bubble = ChatBubble::new("Hello!", MessageType::User);
div().child(bubble.render())

// ビルダーパターン
let bubble = ChatBubbleBuilder::new("AI response")
    .assistant()
    .build();
```

## フォーカス問題の修正

### 問題

入力欄に`track_focus`が設定されておらず、キーボード入力を受け取れませんでした。

### 解決策

```rust
// Before: track_focusがinput_rowに設定されていた
let input_row = div()
    .v_flex()
    .track_focus(&self.focus_handle)  // ❌ ここではなく
    .child(input_area);

// After: input_area自体にtrack_focusを設定
let input_area = div()
    .id("input-area")
    .track_focus(&self.focus_handle)  // ✅ ここに設定
    .on_mouse_down(MouseButton::Left, cx.listener(|_view, _event, window, cx| {
        cx.focus_self(window);
    }))
    .child(input_text_display);
```

## アーキテクチャの改善

### Before

```
neko-assistant/
└── src/
    └── gui/
        └── chat.rs  (600+ lines)
            ├── MessageType enum
            ├── ChatMessage struct
            ├── 手動でChatBubble描画
            ├── 手動でTextInput実装
            └── EntityInputHandler実装
```

### After

```
crates/
├── ui-utils/          # 低レベルユーティリティ
│   ├── TextInputState
│   ├── ScrollManager
│   └── impl_entity_input_handler! macro
│
└── neko-ui/           # 高レベルコンポーネント
    └── ChatBubble     # 再利用可能なBubble

neko-assistant/
└── src/
    └── gui/
        └── chat.rs    (400+ lines, -33% 削減)
            └── コンポーネントの組み合わせのみ
```

## メリット

1. **コードの再利用性**
   - 他のビューでも同じコンポーネントを使用可能
   - プラグイン開発者も利用できる

2. **保守性の向上**
   - コンポーネントごとにテスト可能
   - バグ修正が一箇所で済む

3. **可読性の向上**
   - chat.rsが約200行削減（33%減）
   - 責務が明確に分離

4. **拡張性**
   - 新しいコンポーネントを簡単に追加
   - 既存コンポーネントのカスタマイズが容易

## ファイル構成

```
crates/neko-ui/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    └── chat_bubble.rs
```

## 今後の拡張候補

- Button コンポーネント
- Dropdown/Select コンポーネント
- Modal/Dialog コンポーネント
- ToolBar コンポーネント
- Settings パネルコンポーネント

## テスト

```powershell
# neko-uiのビルド
cargo build -p neko-ui

# アプリ起動
cargo run -p neko-assistant

# 確認項目
# ✅ 起動時に入力欄にフォーカス
# ✅ 入力欄をクリックするとフォーカス移動
# ✅ 日本語入力（IME）が動作
# ✅ Ctrl+Enter/Enterで送信
# ✅ チャットバブルが正しく表示
# ✅ 色分けが正しい（青/緑/グレー/赤）
```

## 関連ドキュメント

- `crates/neko-ui/README.md` - コンポーネントの使用方法
- `crates/ui-utils/README.md` - 低レベルユーティリティ
- `docs/development/ui-utils-extraction.md` - ui-utils作成の経緯
