//! チャット会話履歴管理
//!
//! 会話のメッセージを永続化・読み込みする機能を提供します。

mod conversation;
mod manager;
mod message;

pub use conversation::{Conversation, ConversationMetadata};
pub use manager::ConversationManager;
pub use message::{Message, MessageRole};

#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    #[error("Conversation not found: {0}")]
    NotFound(String),
    
    #[error("Invalid conversation data: {0}")]
    InvalidData(String),
}

pub type Result<T> = std::result::Result<T, HistoryError>;
