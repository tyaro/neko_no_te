# neko-ui

Neko Assistant用のカスタムUIコンポーネントライブラリ

## 概要

このクレートは、Neko Assistantアプリケーションで使用する再利用可能なUIコンポーネントを提供します。

## コンポーネント

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

- **再利用可能**: 複数のビューで同じコンポーネントを使用
- **カスタマイズ可能**: プロパティで外観をカスタマイズ
- **Pure Rust**: GPUIベースの完全なRust実装

## 依存関係

- `gpui 0.2.2`: GPUIフレームワーク
- `gpui-component 0.4.2`: 基本コンポーネント

## ライセンス

MIT
