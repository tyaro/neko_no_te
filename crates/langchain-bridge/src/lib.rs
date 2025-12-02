//! LangChain-rust 統合ブリッジ
//!
//! このクレートは langchain-rust を neko-assistant のアーキテクチャに統合します。
//! 既存の chat-engine インターフェースと互換性のある API を提供します。

use anyhow::Result;
use langchain_rust::{
    agent::{AgentExecutor, ConversationalAgent, ConversationalAgentBuilder},
    chain::{builder::ConversationalChainBuilder, Chain},
    language_models::llm::LLM,
    llm::ollama::client::Ollama,
    memory::SimpleMemory,
    prompt_args,
    tools::Tool,
};
use std::sync::Arc;
use tokio::sync::Mutex;

const JAPANESE_INSTRUCTION: &str = r"あなたは日本語で回答するAIアシスタントです。ツール呼び出し結果や引用した数値があれば、それらを尊重しつつ自然な日本語で簡潔にまとめてください。";

/// LangChain ベースのチャットエンジン
pub struct LangChainEngine {
    ollama: Ollama,
    _base_url: String,
    _model: String,
}

impl LangChainEngine {
    /// 新しい LangChain エンジンを作成
    pub fn new(base_url: &str, model: &str) -> Self {
        let ollama = Ollama::default().with_model(model);

        Self {
            ollama,
            _base_url: base_url.to_string(),
            _model: model.to_string(),
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
                "input" => format!("{}\n\nユーザー入力:\n{}", JAPANESE_INSTRUCTION, message),
            })
            .await?;

        Ok(response)
    }

    /// ストリーミング応答（簡易版）
    pub async fn send_message_simple(&mut self, message: &str) -> Result<String> {
        let prompt = format!("{}\n\nユーザー入力:\n{}", JAPANESE_INSTRUCTION, message);
        let response = self.ollama.invoke(&prompt).await?;
        Ok(response)
    }

    /// 会話履歴をクリア（将来実装）
    pub fn clear_history(&mut self) {
        // SimpleMemory は内部状態を持たないため、何もしない
    }
}

type AgentExecutorInner = AgentExecutor<ConversationalAgent>;

/// LangChain の Tool ベースエージェント
#[derive(Clone)]
pub struct LangChainToolAgent {
    executor: Arc<tokio::sync::Mutex<AgentExecutorInner>>,
}

impl LangChainToolAgent {
    pub fn new(model: &str, tools: Vec<Arc<dyn Tool>>) -> Result<Self> {
        let llm = Ollama::default().with_model(model);
        let memory = SimpleMemory::new();

        let mut builder = ConversationalAgentBuilder::new();
        if !tools.is_empty() {
            builder = builder.tools(&tools);
        }

        let agent = builder.build(llm)?;
        let executor = AgentExecutor::from_agent(agent).with_memory(memory.into());

        Ok(Self {
            executor: Arc::new(tokio::sync::Mutex::new(executor)),
        })
    }

    pub async fn invoke(&self, input: &str) -> Result<String> {
        let vars = prompt_args! {
            "input" => format!("{}\n\nユーザー入力:\n{}", JAPANESE_INSTRUCTION, input),
        };

        let executor = self.executor.lock().await;
        let output = executor.invoke(vars).await?;
        Ok(output)
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
