use crate::{ChatError, Message};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// チャットセッション（メッセージ履歴の集合）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub title: Option<String>,
    pub messages: Vec<Message>,
}

impl ChatSession {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            title: None,
            messages: Vec::new(),
        }
    }
    
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }
    
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = chrono::Utc::now();
    }
    
    /// セッションをJSONファイルに保存
    pub fn save_to_file(&self, path: &Path) -> Result<(), ChatError> {
        // 親ディレクトリを作成
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        
        Ok(())
    }
    
    /// JSONファイルからセッションを読み込み
    pub fn load_from_file(path: &Path) -> Result<Self, ChatError> {
        let json = fs::read_to_string(path)?;
        let session: ChatSession = serde_json::from_str(&json)?;
        Ok(session)
    }
    
    /// セッションディレクトリ内のすべてのセッションを一覧取得
    pub fn list_sessions(session_dir: &Path) -> Result<Vec<SessionInfo>, ChatError> {
        if !session_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut sessions = Vec::new();
        
        for entry in fs::read_dir(session_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                // ファイルを読み込んでメタデータを取得
                if let Ok(session) = Self::load_from_file(&path) {
                    sessions.push(SessionInfo {
                        id: session.id,
                        file_path: path,
                        created_at: session.created_at,
                        updated_at: session.updated_at,
                        title: session.title,
                        message_count: session.messages.len(),
                    });
                }
            }
        }
        
        // 更新日時でソート（新しい順）
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        Ok(sessions)
    }
}

impl Default for ChatSession {
    fn default() -> Self {
        Self::new()
    }
}

/// セッション情報（一覧表示用）
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub file_path: PathBuf,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub title: Option<String>,
    pub message_count: usize,
}
