//! Model provider abstraction for multiple LLM backends.
//!
//! This crate defines a small async trait `ModelProvider` and common types
//! so the rest of the application can integrate with different model
//! providers (local Ollama, remote GPT endpoints, GitHub Copilot, etc.)
//! via a consistent interface.

use async_trait::async_trait;
use serde::Deserialize;

pub mod error;
pub use error::ProviderError;

/// Result of a generate call.
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateResult {
    /// Raw textual output from the provider.
    pub text: String,
    /// Optional structured output if provider returned JSON-like content.
    pub structured: Option<serde_json::Value>,
}

/// Async trait that model providers must implement.
#[async_trait]
pub trait ModelProvider: Send + Sync + 'static {
    /// Human-readable provider name (e.g. "ollama", "openai").
    fn name(&self) -> &str;

    /// Lightweight health check.
    async fn health(&self) -> Result<bool, ProviderError>;

    /// Generate text for the given model and prompt.
    async fn generate(&self, model: &str, prompt: &str) -> Result<GenerateResult, ProviderError>;
}

#[cfg(feature = "ollama-impl")]
pub mod ollama_impl {
    //! A thin adapter that implements `ModelProvider` using the `ollama-client` crate.

    use super::*;
    use crate::ProviderError;

    use ollama_client::OllamaClient;

    pub struct OllamaProvider {
        client: OllamaClient,
        name: String,
    }

    impl OllamaProvider {
        pub fn new(base_url: &str) -> Result<Self, url::ParseError> {
            let client = OllamaClient::new(base_url)?;
            Ok(Self { client, name: "ollama".to_string() })
        }
        
        /// ストリーミングで生成（コールバックで部分応答を受け取る）
        pub async fn generate_stream<F>(
            &self,
            model: &str,
            prompt: &str,
            callback: F,
        ) -> Result<GenerateResult, ProviderError>
        where
            F: FnMut(&str),
        {
            let text = self.client
                .generate_stream(model, prompt, callback)
                .await
                .map_err(|e| ProviderError::Http(e.to_string()))?;
            
            let structured = match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(v) => Some(v),
                Err(_) => None,
            };
            
            Ok(GenerateResult { text, structured })
        }
    }

    #[async_trait]
    impl ModelProvider for OllamaProvider {
        fn name(&self) -> &str {
            &self.name
        }

        async fn health(&self) -> Result<bool, ProviderError> {
            self.client.health().await.map_err(|e| ProviderError::Http(e.to_string()))
        }

        async fn generate(&self, model: &str, prompt: &str) -> Result<GenerateResult, ProviderError> {
            let text = self.client.generate(model, prompt).await.map_err(|e| ProviderError::Http(e.to_string()))?;
            // Try to parse structured JSON if possible, otherwise leave None.
            let structured = match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(v) => Some(v),
                Err(_) => None,
            };
            Ok(GenerateResult { text, structured })
        }
    }
}
