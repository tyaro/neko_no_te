//! 会話管理マネージャー

use std::fs;
use std::path::{Path, PathBuf};
use crate::{Conversation, ConversationMetadata, HistoryError};

/// 会話マネージャー
pub struct ConversationManager {
    storage_dir: PathBuf,
}

impl ConversationManager {
    /// 新しいマネージャーを作成
    /// 
    /// # Arguments
    /// * `storage_dir` - 会話データの保存ディレクトリ
    pub fn new(storage_dir: impl AsRef<Path>) -> Result<Self, HistoryError> {
        let storage_dir = storage_dir.as_ref().to_path_buf();
        
        // ディレクトリが存在しなければ作成
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)
                .map_err(|e| HistoryError::Io(e))?;
        }
        
        Ok(Self { storage_dir })
    }
    
    /// デフォルトの保存ディレクトリを取得
    pub fn default_storage_dir() -> Result<PathBuf, HistoryError> {
        let home = dirs::home_dir()
            .ok_or_else(|| HistoryError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Home directory not found"
            )))?;
        
        Ok(home.join(".neko-assistant").join("conversations"))
    }
    
    /// 会話を保存
    pub fn save(&self, conversation: &Conversation) -> Result<(), HistoryError> {
        let path = self.conversation_path(&conversation.id);
        let json = serde_json::to_string_pretty(conversation)
            .map_err(|e| HistoryError::Serialization(e.to_string()))?;
        
        fs::write(path, json)
            .map_err(|e| HistoryError::Io(e))?;
        
        Ok(())
    }
    
    /// 会話を読み込み
    pub fn load(&self, id: &str) -> Result<Conversation, HistoryError> {
        let path = self.conversation_path(id);
        
        if !path.exists() {
            return Err(HistoryError::NotFound(id.to_string()));
        }
        
        let json = fs::read_to_string(path)
            .map_err(|e| HistoryError::Io(e))?;
        
        let conversation = serde_json::from_str(&json)
            .map_err(|e| HistoryError::Deserialization(e.to_string()))?;
        
        Ok(conversation)
    }
    
    /// 会話を削除
    pub fn delete(&self, id: &str) -> Result<(), HistoryError> {
        let path = self.conversation_path(id);
        
        if !path.exists() {
            return Err(HistoryError::NotFound(id.to_string()));
        }
        
        fs::remove_file(path)
            .map_err(|e| HistoryError::Io(e))?;
        
        Ok(())
    }
    
    /// すべての会話のメタデータを取得
    pub fn list_metadata(&self) -> Result<Vec<ConversationMetadata>, HistoryError> {
        let mut metadata_list = Vec::new();
        
        let entries = fs::read_dir(&self.storage_dir)
            .map_err(|e| HistoryError::Io(e))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| HistoryError::Io(e))?;
            let path = entry.path();
            
            // JSONファイルのみ対象
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            
            // ファイル名から会話IDを取得
            if let Some(id) = path.file_stem().and_then(|s| s.to_str()) {
                if let Ok(conversation) = self.load(id) {
                    metadata_list.push(conversation.to_metadata());
                }
            }
        }
        
        // 更新日時でソート（新しい順）
        metadata_list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        
        Ok(metadata_list)
    }
    
    /// 会話のファイルパスを取得
    fn conversation_path(&self, id: &str) -> PathBuf {
        self.storage_dir.join(format!("{}.json", id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{Message, MessageRole};
    use tempfile::tempdir;
    
    #[test]
    fn test_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let manager = ConversationManager::new(temp_dir.path()).unwrap();
        
        let mut conversation = Conversation::new("Test Conversation");
        conversation.add_message(Message::new(
            MessageRole::User,
            "Hello".to_string(),
        ));
        
        // 保存
        manager.save(&conversation).unwrap();
        
        // 読み込み
        let loaded = manager.load(&conversation.id).unwrap();
        assert_eq!(loaded.id, conversation.id);
        assert_eq!(loaded.title, conversation.title);
        assert_eq!(loaded.messages.len(), 1);
    }
    
    #[test]
    fn test_delete() {
        let temp_dir = tempdir().unwrap();
        let manager = ConversationManager::new(temp_dir.path()).unwrap();
        
        let conversation = Conversation::new("Test");
        manager.save(&conversation).unwrap();
        
        // 削除
        manager.delete(&conversation.id).unwrap();
        
        // 読み込み失敗することを確認
        assert!(manager.load(&conversation.id).is_err());
    }
    
    #[test]
    fn test_list_metadata() {
        let temp_dir = tempdir().unwrap();
        let manager = ConversationManager::new(temp_dir.path()).unwrap();
        
        // 複数の会話を作成
        for i in 0..3 {
            let conversation = Conversation::new(format!("Conversation {}", i));
            manager.save(&conversation).unwrap();
        }
        
        // メタデータ一覧を取得
        let metadata_list = manager.list_metadata().unwrap();
        assert_eq!(metadata_list.len(), 3);
    }
}
