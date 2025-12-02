//! 会話型定義

use crate::message::Message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 会話
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<Message>,
}

impl Conversation {
    /// 新しい会話を作成
    pub fn new(title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.into(),
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
        }
    }

    /// メッセージを追加
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    /// 最新のメッセージを取得
    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    /// メタデータに変換（一覧表示用）
    pub fn to_metadata(&self) -> ConversationMetadata {
        ConversationMetadata {
            id: self.id.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            message_count: self.messages.len(),
        }
    }
}

/// 会話メタデータ（一覧表示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
}
