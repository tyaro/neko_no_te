//! チャットエンジン
//!
//! ModelProvider と ModelAdapter を使用して会話を管理します。

use chrono::{DateTime, Utc};
use model_adapter::{ModelAdapter, ModelProvider, ToolSpec};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub mod error;
pub mod session;

pub use error::ChatError;
pub use session::{ChatSession, SessionInfo};

/// メッセージのロール
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// 単一のメッセージ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: Role,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Message {
    pub fn new(role: Role, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content,
            timestamp: Utc::now(),
            metadata: None,
        }
    }
    
    pub fn user(content: String) -> Self {
        Self::new(Role::User, content)
    }
    
    pub fn assistant(content: String) -> Self {
        Self::new(Role::Assistant, content)
    }
    
    pub fn system(content: String) -> Self {
        Self::new(Role::System, content)
    }
}

/// チャットエンジン
pub struct ChatEngine {
    provider: Arc<dyn ModelProvider>,
    adapter: Arc<dyn ModelAdapter>,
    model: String,
    history: Vec<Message>,
    max_history: usize,
    system_prompt: Option<String>,
}

impl ChatEngine {
    pub fn new(
        provider: Arc<dyn ModelProvider>,
        adapter: Arc<dyn ModelAdapter>,
        model: String,
    ) -> Self {
        Self {
            provider,
            adapter,
            model,
            history: Vec::new(),
            max_history: 100,
            system_prompt: None,
        }
    }
    
    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = Some(prompt);
        self
    }
    
    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }
    
    /// メッセージ履歴を取得
    pub fn get_history(&self) -> &[Message] {
        &self.history
    }
    
    /// メッセージ履歴をクリア
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
    
    /// セッションを保存
    pub fn save_session(&self, session_dir: &std::path::Path, session_id: Option<&str>) -> Result<String, ChatError> {
        let mut session = ChatSession::new();
        
        if let Some(id) = session_id {
            session.id = id.to_string();
        }
        
        // メッセージをセッションにコピー
        for msg in &self.history {
            session.add_message(msg.clone());
        }
        
        // ファイルパスを生成
        let file_name = format!("{}.json", session.id);
        let file_path = session_dir.join(file_name);
        
        // 保存
        session.save_to_file(&file_path)?;
        
        Ok(session.id)
    }
    
    /// セッションから履歴を読み込み
    pub fn load_session(&mut self, session_path: &std::path::Path) -> Result<(), ChatError> {
        let session = ChatSession::load_from_file(session_path)?;
        self.history = session.messages;
        Ok(())
    }
    
    /// ユーザーメッセージを送信して応答を取得
    pub async fn send_message(&mut self, user_input: &str) -> Result<String, ChatError> {
        self.send_message_with_tools(user_input, None).await
    }
    
    /// ツール付きメッセージを送信
    pub async fn send_message_with_tools(
        &mut self,
        user_input: &str,
        tools: Option<&[ToolSpec]>,
    ) -> Result<String, ChatError> {
        // ユーザーメッセージを履歴に追加
        let user_msg = Message::user(user_input.to_string());
        self.history.push(user_msg);
        
        // プロバイダを通じて応答を取得
        let result = self.adapter
            .invoke(
                self.provider.as_ref(),
                &self.model,
                user_input,
                tools,
            )
            .await?;
        
        // アシスタントの応答を履歴に追加
        let assistant_msg = Message::assistant(result.text.clone());
        self.history.push(assistant_msg);
        
        // 履歴が長すぎる場合はトリミング（システムメッセージは保持）
        self.trim_history();
        
        Ok(result.text)
    }
    
    /// 履歴をトリミング（古いメッセージを削除）
    fn trim_history(&mut self) {
        if self.history.len() > self.max_history {
            // システムメッセージを保持
            let system_messages: Vec<_> = self.history
                .iter()
                .filter(|m| m.role == Role::System)
                .cloned()
                .collect();
            
            // 最新のメッセージを保持
            let recent_count = self.max_history - system_messages.len();
            let recent_messages: Vec<_> = self.history
                .iter()
                .rev()
                .filter(|m| m.role != Role::System)
                .take(recent_count)
                .cloned()
                .collect();
            
            // 再構成
            self.history = system_messages;
            self.history.extend(recent_messages.into_iter().rev());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use model_adapter::{GenerateResult, Phi4MiniAdapter};
    use model_provider::ProviderError;
    
    struct MockProvider;
    
    #[async_trait::async_trait]
    impl ModelProvider for MockProvider {
        fn name(&self) -> &str {
            "mock"
        }
        
        async fn health(&self) -> Result<bool, ProviderError> {
            Ok(true)
        }
        
        async fn generate(&self, _model: &str, prompt: &str) -> Result<GenerateResult, ProviderError> {
            Ok(GenerateResult {
                text: format!("Response to: {}", prompt),
                structured: None,
            })
        }
    }
    
    #[tokio::test]
    async fn test_send_message() {
        let provider = Arc::new(MockProvider);
        let adapter = Arc::new(Phi4MiniAdapter::new());
        let mut engine = ChatEngine::new(provider, adapter, "phi4-mini:3.8b".to_string());
        
        let response = engine.send_message("Hello").await.unwrap();
        assert!(!response.is_empty());
        
        // 履歴確認
        assert_eq!(engine.get_history().len(), 2); // user + assistant
        assert_eq!(engine.get_history()[0].role, Role::User);
        assert_eq!(engine.get_history()[1].role, Role::Assistant);
    }
    
    #[tokio::test]
    async fn test_history_trimming() {
        let provider = Arc::new(MockProvider);
        let adapter = Arc::new(Phi4MiniAdapter::new());
        let mut engine = ChatEngine::new(provider, adapter, "test".to_string())
            .with_max_history(4);
        
        // 複数メッセージを送信
        for i in 0..5 {
            engine.send_message(&format!("Message {}", i)).await.unwrap();
        }
        
        // 履歴が max_history 以下に保たれている
        assert!(engine.get_history().len() <= 4);
    }
}
