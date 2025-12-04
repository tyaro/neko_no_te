//! Phi-4-mini-instruct adapter plugin.
//!
//! Microsoft の [Phi-4-mini-instruct](https://huggingface.co/microsoft/Phi-4-mini-instruct)
//! は `<|role|>` トークンで会話を区切るシンプルなチャットフォーマットと、
//! `<|tool|> ... <|/tool|>` ブロックによるネイティブな function calling を提供します。
//! このプラグインは `ModelAdapter` を実装し、`chat-core` / `neko-assistant`
//! がプラグイン経由でフォーマット差分を吸収できるようにします。

use async_trait::async_trait;
use model_adapter::{ModelAdapter, ToolSpec};
use model_provider::{GenerateResult, ModelProvider, ProviderError};
use serde_json::json;

const DEFAULT_SYSTEM_PROMPT: &str = "You are a helpful assistant.";

/// Adapter implementation for Phi-4-mini-instruct.
pub struct Phi4MiniAdapter;

impl Phi4MiniAdapter {
    pub fn new() -> Self {
        Self
    }

    fn format_tools(tools: &[ToolSpec]) -> String {
        let tool_defs: Vec<_> = tools
            .iter()
            .map(|tool| {
                json!({
                    "name": tool.name,
                    "description": tool
                        .description
                        .as_deref()
                        .unwrap_or(""),
                    "parameters": tool.schema.clone().unwrap_or_else(|| json!({})),
                })
            })
            .collect();

        format!(
            "<|tool|>\n{}\n<|/tool|>",
            serde_json::to_string_pretty(&tool_defs).unwrap()
        )
    }

    fn format_prompt(
        system_prompt: Option<&str>,
        user_prompt: &str,
        tools: Option<&[ToolSpec]>,
    ) -> String {
        let mut prompt = String::new();
        let mut system = system_prompt.unwrap_or(DEFAULT_SYSTEM_PROMPT).to_string();

        if let Some(ts) = tools {
            if !ts.is_empty() {
                system.push_str(" with access to these tools.\n");
                system.push_str(&Self::format_tools(ts));
            }
        }

        prompt.push_str("<|system|>\n");
        prompt.push_str(&system);
        prompt.push_str("<|end|>\n");

        prompt.push_str("<|user|>\n");
        prompt.push_str(user_prompt);
        prompt.push_str("<|end|>\n");

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
            "phi4-mini".to_string(),
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
    async fn formats_prompt_without_tools() {
        let adapter = Phi4MiniAdapter::new();
        let provider = DummyProvider;
        let res = adapter
            .invoke(&provider, "phi4-mini:3.8b", "こんにちは", None)
            .await
            .unwrap();

        assert!(res.text.contains("<|system|>"));
        assert!(res.text.contains("<|user|>"));
        assert!(res.text.contains("こんにちは"));
    }

    #[tokio::test]
    async fn formats_prompt_with_tools() {
        let adapter = Phi4MiniAdapter::new();
        let provider = DummyProvider;
        let tools = vec![ToolSpec::new("get_weather", "Fetch weather")];
        let res = adapter
            .invoke(&provider, "Phi-4-mini-instruct", "Use a tool", Some(&tools))
            .await
            .unwrap();

        assert!(res.text.contains("<|tool|>"));
        assert!(res.text.contains("get_weather"));
        assert!(res.text.contains("Use a tool"));
    }
}
