//! チャットバブルコンポーネント

use gpui::*;

/// メッセージタイプ
#[derive(Clone, Debug, PartialEq)]
pub enum MessageType {
    User,
    Assistant,
    System,
    Error,
}

/// チャットバブルコンポーネント
#[derive(Clone)]
pub struct ChatBubble {
    content: SharedString,
    msg_type: MessageType,
}

impl ChatBubble {
    /// 新しいチャットバブルを作成
    pub fn new(content: impl Into<SharedString>, msg_type: MessageType) -> Self {
        Self {
            content: content.into(),
            msg_type,
        }
    }
    
    /// メッセージタイプに応じた背景色を取得
    fn background_color(&self) -> Rgba {
        match self.msg_type {
            MessageType::User => rgb(0x3b82f6),      // 青
            MessageType::Assistant => rgb(0x10b981), // 緑
            MessageType::System => rgb(0x6b7280),    // グレー
            MessageType::Error => rgb(0xef4444),     // 赤
        }
    }
    
    /// メッセージタイプに応じたテキスト色を取得
    fn text_color(&self) -> Rgba {
        rgb(0xffffff) // 白
    }
    
    /// レンダリング
    pub fn render(&self) -> impl IntoElement {
        div()
            .p_3()
            .rounded(px(12.0))
            .bg(self.background_color())
            .text_color(self.text_color())
            .child(self.content.clone())
    }
}

/// ChatBubbleのビルダーパターン
pub struct ChatBubbleBuilder {
    content: String,
    msg_type: MessageType,
}

impl ChatBubbleBuilder {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            msg_type: MessageType::User,
        }
    }
    
    pub fn user(mut self) -> Self {
        self.msg_type = MessageType::User;
        self
    }
    
    pub fn assistant(mut self) -> Self {
        self.msg_type = MessageType::Assistant;
        self
    }
    
    pub fn system(mut self) -> Self {
        self.msg_type = MessageType::System;
        self
    }
    
    pub fn error(mut self) -> Self {
        self.msg_type = MessageType::Error;
        self
    }
    
    pub fn build(self) -> ChatBubble {
        ChatBubble::new(self.content, self.msg_type)
    }
}
