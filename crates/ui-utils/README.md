# ui-utils

UI共通ユーティリティクレート

## 概要

複数のビューで共有されるUI機能を提供します：

- **テキスト入力とIME対応** (`text_input.rs`)
  - `TextInputState`: テキスト入力の状態管理
  - `impl_entity_input_handler!`: EntityInputHandlerの標準実装マクロ
  
- **スクロール機能** (`scroll_utils.rs`)
  - `ScrollManager`: スクロール管理のヘルパー

## 使用例

### テキスト入力

```rust
use ui_utils::{TextInputState, TextInputHandler, impl_entity_input_handler};

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

// EntityInputHandlerの実装を自動生成
impl_entity_input_handler!(MyView);
```

### スクロール管理

```rust
use ui_utils::ScrollManager;

struct MyView {
    scroll_manager: ScrollManager,
    // ...
}

impl Render for MyView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // スクロール更新
        self.scroll_manager.update();
        
        div()
            .overflow_y_scroll()
            .track_scroll(self.scroll_manager.handle())
            // ...
    }
}
```

## ライセンス

MIT
