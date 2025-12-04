# UI共通機能の切り出し

## 概要

IME対応とスクロール機能を共通クレート `ui-utils` に切り出しました。これにより、他のプラグインやビューでも同じ機能を簡単に再利用できます。

> **2025-12-04 メモ**: Phase 5 時点では `neko-ui` 側からの依存は一旦解除し、
> `chat_input` などは `gpui-component::InputState` ベースに集約しました。今後
> IME 特化 UI を復活させる場合は、このクレートの TextInput 系ユーティリティを
> 参照してください。

## 実装内容

### 1. `ui-utils` クレートの作成

新しいクレート `crates/ui-utils` を作成し、以下の機能を提供：

- **TextInputState**: テキスト入力の状態管理（IME対応）
- **TextInputHandler**: トレイト定義
- **impl_entity_input_handler!**: EntityInputHandler自動実装マクロ
- **ScrollManager**: スクロール管理のヘルパー

### 2. chat.rs のリファクタリング

#### Before
```rust
struct ChatView {
    input_text: Rc<RefCell<String>>,
    selected_range: Rc<RefCell<Range<usize>>>,
    marked_range: Rc<RefCell<Option<Range<usize>>>>,
    scroll_to_bottom: Rc<RefCell<bool>>,
    scroll_handle: ScrollHandle,
    // ... 手動でEntityInputHandlerを実装
}
```

#### After
```rust
struct ChatView {
    text_state: TextInputState,
    scroll_manager: ScrollManager,
    // ...
}

impl TextInputHandler for ChatView {
    fn text_input_state(&self) -> &TextInputState {
        &self.text_state
    }
    
    fn text_input_state_mut(&mut self) -> &mut TextInputState {
        &mut self.text_state
    }
}

// 自動実装
impl_entity_input_handler!(ChatView);
```

### 3. 主な変更点

#### TextInputState
- `input_text`, `selected_range`, `marked_range` を1つの構造体に統合
- `clear()`, `text()`, `replace_text_in_range()` などのメソッドを提供

#### ScrollManager
- `scroll_handle` と `scroll_to_bottom` フラグを1つにまとめ
- `mark_scroll_to_bottom()` と `update()` でシンプルな API

#### マクロ
- `impl_entity_input_handler!` で EntityInputHandler の8メソッドを自動生成
- `TextInputHandler` トレイトを実装するだけで使用可能

### 4. 使用例

新しいビューでIME対応入力を追加する場合：

```rust
use ui_utils::{TextInputState, TextInputHandler};

struct MyView {
    text_state: TextInputState,
    // ...
}

impl TextInputHandler for MyView {
    fn text_input_state(&self) -> &TextInputState {
        &self.text_state
    }
    
    fn text_input_state_mut(&mut self) -> &mut TextInputState {
        &mut self.text_state
    }
}

impl_entity_input_handler!(MyView);
```

## メリット

1. **コードの再利用**: 他のビューで同じIME機能を簡単に実装
2. **保守性向上**: 共通コードを1箇所で管理
3. **シンプルな API**: 複雑な内部実装を隠蔽
4. **プラグイン対応**: プラグイン開発者も使いやすい

## ファイル構成

```
crates/ui-utils/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs
    ├── text_input.rs      # IME対応
    └── scroll_utils.rs    # スクロール管理
```

## テスト

```powershell
# ビルド確認
cargo build --workspace

# アプリ起動
cargo run -p neko-assistant

# 確認項目
# - 起動時に入力欄にフォーカスがある
# - 入力欄をクリックするとフォーカスが移動
# - 日本語入力（IME）が動作
# - チャット後に自動スクロール
```

## 関連ドキュメント

- `docs/design/ime-and-multiline-input.md` - IME実装の詳細設計
- `crates/ui-utils/README.md` - クレートの使用方法
- `docs/development/coding-guidelines.md` - モジュール分割の原則
