//! Neko Assistant カスタムUIコンポーネント
//!
//! このクレートはNeko Assistantで使用するカスタムUIコンポーネントを提供します：
//! - ChatBubble: チャットメッセージの表示
//! - ChatInput: IME対応の複数行入力（gpui-component Input のラッパー）
//! - その他の共通UIコンポーネント

pub mod chat_bubble;
pub mod chat_input;
pub mod chat_input_panel;
pub mod chat_main_panel;
pub mod chat_message_list;
pub mod chat_messages_panel;
pub mod chat_sidebar;
pub mod chat_toolbar;
pub mod chat_workspace;
pub mod mcp_status_panel;
pub mod model_selector;
pub mod model_selector_row;
pub mod scratchpad_console;

pub use chat_bubble::{ChatBubble, MessageType};
pub use chat_input::{ChatInput, SendKeyConfig};
pub use chat_input_panel::chat_input_panel;
pub use chat_main_panel::chat_main_panel;
pub use chat_message_list::{chat_message_list, ChatMessageRow};
pub use chat_messages_panel::chat_messages_panel;
pub use chat_sidebar::{chat_sidebar, ChatSidebarItem};
pub use chat_toolbar::chat_toolbar;
pub use chat_workspace::chat_workspace;
pub use mcp_status_panel::{mcp_status_panel, McpServerItem, McpServerStatusBadge, McpToolItem};
pub use model_selector::{model_selector, ModelPreset};
pub use model_selector_row::model_selector_row;
pub use scratchpad_console::{scratchpad_console, ConsoleLogEntry};
