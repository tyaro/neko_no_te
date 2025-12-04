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

impl ToolSpec {
    /// 簡易的なツール定義
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: Some(description.to_string()),
            schema: None,
        }
    }

    /// パラメータ付きツール定義
    pub fn with_parameters(name: &str, description: &str, parameters: serde_json::Value) -> Self {
        Self {
            name: name.to_string(),
            description: Some(description.to_string()),
            schema: Some(parameters),
        }
    }
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

impl Default for Llama3DefaultAdapter {
    fn default() -> Self {
        Self::new()
    }
}

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

/// Adapter for Phi-4-mini-instruct model.
///
/// This adapter uses Phi-4-mini-instruct's specific chat format:
/// `<|system|>...<|end|><|user|>...<|end|><|assistant|>`
///
/// It also supports native function calling with the `<|tool|>...</|tool|>` format.
pub struct Phi4MiniAdapter;

impl Default for Phi4MiniAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Phi4MiniAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Format tools in Phi-4-mini-instruct format
    fn format_tools(tools: &[ToolSpec]) -> String {
        let tools_json: Vec<serde_json::Value> = tools
            .iter()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name,
                    "description": tool.description.as_ref().unwrap_or(&String::new()),
                    "parameters": tool.schema.as_ref().unwrap_or(&serde_json::json!({}))
                })
            })
            .collect();

        format!(
            "<|tool|>\n{}\n<|/tool|>",
            serde_json::to_string_pretty(&tools_json).unwrap()
        )
    }

    /// Format prompt in Phi-4-mini-instruct chat format
    fn format_prompt(
        system: Option<&str>,
        user_prompt: &str,
        tools: Option<&[ToolSpec]>,
    ) -> String {
        let mut prompt = String::new();

        // System message with optional tools
        let mut system_msg = system.unwrap_or("You are a helpful assistant.").to_string();
        if let Some(t) = tools {
            if !t.is_empty() {
                system_msg.push_str(" with access to these tools.\n");
                system_msg.push_str(&Self::format_tools(t));
            }
        }

        prompt.push_str("<|system|>\n");
        prompt.push_str(&system_msg);
        prompt.push_str("<|end|>\n");

        // User message
        prompt.push_str("<|user|>\n");
        prompt.push_str(user_prompt);
        prompt.push_str("<|end|>\n");

        // Assistant response starts here
        prompt.push_str("<|assistant|>\n");

        prompt
    }
}

#[async_trait]
impl ModelAdapter for Phi4MiniAdapter {
    fn adapter_name(&self) -> &str {
        "phi4-mini-adapter"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "phi4-mini:3.8b".to_string(),
            "Phi-4-mini-instruct".to_string(),
        ]
    }

    async fn invoke(
        &self,
        provider: &dyn ModelProvider,
        model: &str,
        prompt: &str,
        tools: Option<&[ToolSpec]>,
    ) -> Result<GenerateResult, ProviderError> {
        // Format prompt in Phi-4-mini-instruct format
        let formatted = Self::format_prompt(None, prompt, tools);

        provider.generate(model, &formatted).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use model_provider::GenerateResult;

    struct DummyProvider;
    #[async_trait::async_trait]
    impl model_provider::ModelProvider for DummyProvider {
        fn name(&self) -> &str {
            "dummy"
        }
        async fn health(&self) -> Result<bool, model_provider::ProviderError> {
            Ok(true)
        }
        async fn generate(
            &self,
            _model: &str,
            prompt: &str,
        ) -> Result<GenerateResult, model_provider::ProviderError> {
            Ok(GenerateResult {
                text: format!("echo: {}", prompt),
                structured: None,
            })
        }
    }

    #[tokio::test]
    async fn llama_adapter_invokes_provider() {
        let adapter = Llama3DefaultAdapter::new();
        let provider = DummyProvider;
        let tool = ToolSpec {
            name: "t1".into(),
            description: Some("d".into()),
            schema: None,
        };
        let res = adapter
            .invoke(&provider, "llama3.1:8b", "hello", Some(&[tool]))
            .await
            .unwrap();
        assert!(res.text.contains("hello"));
    }

    #[tokio::test]
    async fn phi4_adapter_formats_correctly() {
        let adapter = Phi4MiniAdapter::new();
        let provider = DummyProvider;

        // Test without tools
        let res = adapter
            .invoke(&provider, "phi4-mini:3.8b", "hello", None)
            .await
            .unwrap();
        assert!(res.text.contains("<|system|>"));
        assert!(res.text.contains("<|user|>"));
        assert!(res.text.contains("hello"));

        // Test with tools
        let tool = ToolSpec::new("test_tool", "A test tool");
        let res = adapter
            .invoke(&provider, "phi4-mini:3.8b", "use tool", Some(&[tool]))
            .await
            .unwrap();
        assert!(res.text.contains("<|tool|>"));
        assert!(res.text.contains("test_tool"));
    }
}
