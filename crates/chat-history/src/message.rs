//! メッセージ型定義

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// メッセージの役割
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Error,
}

/// チャットメッセージ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Message {
    /// 新しいメッセージを作成
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role,
            content: content.into(),
            timestamp: Utc::now(),
            metadata: None,
        }
    }

    /// メタデータ付きメッセージを作成
    pub fn with_metadata(
        role: MessageRole,
        content: impl Into<String>,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role,
            content: content.into(),
            timestamp: Utc::now(),
            metadata: Some(metadata),
        }
    }
}
