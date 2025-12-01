//! LangChain-rust 統合ブリッジ
//!
//! このクレートは langchain-rust を neko-assistant のアーキテクチャに統合します。
//! 既存の chat-engine インターフェースと互換性のある API を提供します。

use anyhow::Result;
use langchain_rust::{
    chain::{Chain, builder::ConversationalChainBuilder},
    llm::ollama::client::Ollama,
    language_models::llm::LLM,
    memory::SimpleMemory,
    prompt_args,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// LangChain ベースのチャットエンジン
pub struct LangChainEngine {
    ollama: Ollama,
    base_url: String,
    model: String,
}

impl LangChainEngine {
    /// 新しい LangChain エンジンを作成
    pub fn new(base_url: &str, model: &str) -> Self {
        let ollama = Ollama::default()
            .with_model(model);
        
        Self {
            ollama,
            base_url: base_url.to_string(),
            model: model.to_string(),
        }
    }
    
    /// 会話履歴を含むメッセージ送信
    pub async fn send_message(&mut self, message: &str) -> Result<String> {
        let memory = Arc::new(Mutex::new(SimpleMemory::new()));
        let chain = ConversationalChainBuilder::new()
            .llm(self.ollama.clone())
            .memory(memory)
            .build()?;
        
        let response = chain
            .invoke(prompt_args! {
                "input" => message,
            })
            .await?;
        
        Ok(response)
    }
    
    /// ストリーミング応答（簡易版）
    pub async fn send_message_simple(&mut self, message: &str) -> Result<String> {
        let response = self.ollama.invoke(message).await?;
        Ok(response)
    }
    
    /// 会話履歴をクリア（将来実装）
    pub fn clear_history(&mut self) {
        // SimpleMemory は内部状態を持たないため、何もしない
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // CI環境では Ollama が動作しないためスキップ
    async fn test_langchain_engine() {
        let mut engine = LangChainEngine::new("http://localhost:11434", "phi4-mini:3.8b");
        
        let response = engine.send_message("こんにちは").await.unwrap();
        assert!(!response.is_empty());
    }
}
