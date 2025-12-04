//! ModelAdapter for Qwen 3 (4B Instruct) chat format.
//! Qwen 3 uses ChatML-like format with <|im_start|> and <|im_end|> tokens.

use async_trait::async_trait;
use model_adapter::{ModelAdapter, ToolSpec};
use model_provider::{GenerateResult, ModelProvider, ProviderError};

/// ModelAdapter for Qwen 3 (4B Instruct)
pub struct Qwen3Adapter;

impl Default for Qwen3Adapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Qwen3Adapter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ModelAdapter for Qwen3Adapter {
    fn adapter_name(&self) -> &str {
        "qwen3-adapter"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "qwen3:4b".to_string(),
            "qwen3:latest".to_string(),
            "qwen3".to_string(),
            "qwen3:4b-instruct".to_string(),
        ]
    }

    async fn invoke(
        &self,
        provider: &dyn ModelProvider,
        model: &str,
        prompt: &str,
        tools: Option<&[ToolSpec]>,
    ) -> Result<GenerateResult, ProviderError> {
        // Qwen 3 も Ollama 側でチャット形式を処理するため、
        // tools をプロンプトに注入する
        let mut actual_prompt = prompt.to_string();
        if let Some(t) = tools {
            if !t.is_empty() {
                let tools_desc = t
                    .iter()
                    .map(|ts| {
                        let desc = ts.description.as_deref().unwrap_or("(no description)");
                        let schema = ts
                            .schema
                            .as_ref()
                            .map(|s| serde_json::to_string_pretty(s).unwrap_or_default())
                            .unwrap_or_else(|| "(no schema)".to_string());
                        format!("- {}: {}\n  Schema: {}", ts.name, desc, schema)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                actual_prompt = format!(
                    "You have access to the following tools:\n\n{}\n\n\
                     When you need to use a tool, respond with a JSON object: \
                     {{\"tool_name\": \"...\", \"parameters\": {{...}}}}\n\n\
                     User request: {}",
                    tools_desc, prompt
                );
            }
        }

        provider.generate(model, &actual_prompt).await
    }
}
