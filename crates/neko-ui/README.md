# neko-ui

Neko Assistant用のカスタムUIコンポーネントライブラリ

## 概要

このクレートは、Neko Assistantアプリケーションで使用する再利用可能なUIコンポーネントを提供します。

## コンポーネント

### TextInput

IME対応の複数行テキスト入力コンポーネント

```rust
use neko_ui::TextInput;

// ビュー内で使用
struct MyView {
    text_input: View<TextInput>,
}

impl MyView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let text_input = cx.new(|cx| {
            TextInput::new(cx)
                .placeholder("Type a message...")
                .min_height(px(80.0))
                .max_height(px(200.0))
        });
        
        Self { text_input }
    }
}
```

### ChatBubble

チャットメッセージ表示用のバブルコンポーネント

```rust
use neko_ui::{ChatBubble, MessageType};

// メッセージを表示
let bubble = ChatBubble::new("Hello, world!", MessageType::User);
div().child(bubble.render())

// ビルダーパターン
use neko_ui::ChatBubbleBuilder;

let bubble = ChatBubbleBuilder::new("AI response")
    .assistant()
    .build();
```

## 特徴

- **IME対応**: 日本語入力を完全サポート
- **再利用可能**: 複数のビューで同じコンポーネントを使用
- **カスタマイズ可能**: プロパティで外観をカスタマイズ
- **Pure Rust**: GPUIベースの完全なRust実装

## 依存関係

- `gpui 0.2.2`: GPUIフレームワーク
- `gpui-component 0.4.2`: 基本コンポーネント
- `ui-utils`: テキスト入力とスクロールユーティリティ

## ライセンス

MIT
