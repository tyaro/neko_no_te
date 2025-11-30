//! Model adapter trait and a default adapter for `llama3.1:8b`.
//!
//! An adapter encapsulates model-specific call formatting (function-call/tool
//! invocation format) so that the application can support many different
//! model formats via plugins. The crate provides a small trait and a
//! builtin adapter for `llama3.1:8b` that acts as the default.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use model_provider::{GenerateResult, ModelProvider, ProviderError};

/// Representation of a tool/function the model can call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: Option<String>,
    pub schema: Option<serde_json::Value>,
}

/// Async trait implemented by adapters for specific models.
#[async_trait]
pub trait ModelAdapter: Send + Sync + 'static {
    /// Adapter identifier (e.g. "llama3-default").
    fn adapter_name(&self) -> &str;

    /// List of model names this adapter supports (e.g. ["llama3.1:8b"]).
    fn supported_models(&self) -> Vec<String>;

    /// Invoke the provider for the given model using adapter-specific formatting.
    ///
    /// `tools` describes available functions the model may call. Adapters are
    /// responsible for serializing these into the provider-specific payload
    /// (if the provider/model supports function calling).
    async fn invoke(
        &self,
        provider: &dyn ModelProvider,
        model: &str,
        prompt: &str,
        tools: Option<&[ToolSpec]>,
    ) -> Result<GenerateResult, ProviderError>;
}

/// Default adapter for `llama3.1:8b`.
pub struct Llama3DefaultAdapter;

impl Llama3DefaultAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ModelAdapter for Llama3DefaultAdapter {
    fn adapter_name(&self) -> &str {
        "llama3-default"
    }

    fn supported_models(&self) -> Vec<String> {
        vec!["llama3.1:8b".to_string()]
    }

    async fn invoke(
        &self,
        provider: &dyn ModelProvider,
        model: &str,
        prompt: &str,
        tools: Option<&[ToolSpec]>,
    ) -> Result<GenerateResult, ProviderError> {
        // Llama3 default behavior: if tools are provided, append a short
        // serialized description to the prompt so the model can call them.
        let mut actual_prompt = prompt.to_string();

        if let Some(t) = tools {
            if !t.is_empty() {
                let mut tool_descs = vec![];
                for tool in t {
                    let desc = serde_json::json!({
                        "name": tool.name,
                        "description": tool.description,
                        "schema": tool.schema,
                    });
                    tool_descs.push(desc);
                }
                let tools_section = serde_json::to_string(&tool_descs).unwrap_or_default();
                actual_prompt.push_str("\n\nAvailable tools: ");
                actual_prompt.push_str(&tools_section);
            }
        }

        // Delegate to provider. Here we assume the provider expects model
        // name exactly as given (e.g. "llama3.1:8b").
        provider.generate(model, &actual_prompt).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use model_provider::GenerateResult;

    struct DummyProvider;
    #[async_trait::async_trait]
    impl model_provider::ModelProvider for DummyProvider {
        fn name(&self) -> &str { "dummy" }
        async fn health(&self) -> Result<bool, model_provider::ProviderError> { Ok(true) }
        async fn generate(&self, _model: &str, prompt: &str) -> Result<GenerateResult, model_provider::ProviderError> {
            Ok(GenerateResult { text: format!("echo: {}", prompt), structured: None })
        }
    }

    #[tokio::test]
    async fn llama_adapter_invokes_provider() {
        let adapter = Llama3DefaultAdapter::new();
        let provider = DummyProvider;
        let tool = ToolSpec { name: "t1".into(), description: Some("d".into()), schema: None };
        let res = adapter.invoke(&provider, "llama3.1:8b", "hello", Some(&[tool])).await.unwrap();
        assert!(res.text.contains("hello"));
    }
}
