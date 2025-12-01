//! Neko Assistant カスタムUIコンポーネント
//!
//! このクレートはNeko Assistantで使用するカスタムUIコンポーネントを提供します：
//! - ChatBubble: チャットメッセージの表示
//! - ChatInput: IME対応の複数行入力（gpui-component Input のラッパー）
//! - その他の共通UIコンポーネント

pub mod chat_bubble;
pub mod chat_input;

pub use chat_bubble::{ChatBubble, MessageType};
pub use chat_input::{ChatInput, SendKeyConfig};
