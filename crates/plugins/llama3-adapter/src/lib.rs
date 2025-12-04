//! ModelAdapter for Meta Llama 3.1 (8B Instruct) chat format.
//! Llama 3.1 uses special tokens: <|begin_of_text|>, <|start_header_id|>, <|end_header_id|>, <|eot_id|>

use async_trait::async_trait;
use model_adapter::{ModelAdapter, ToolSpec};
use model_provider::{GenerateResult, ModelProvider, ProviderError};

/// ModelAdapter for Meta Llama 3.1 (8B Instruct)
pub struct Llama3Adapter;

impl Default for Llama3Adapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Llama3Adapter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ModelAdapter for Llama3Adapter {
    fn adapter_name(&self) -> &str {
        "llama3-adapter"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "llama3.1:8b".to_string(),
            "llama3.1:latest".to_string(),
            "llama3.1".to_string(),
            "llama3.2:3b".to_string(),
            "llama3.2".to_string(),
            "pakachan/elyza-llama3-8b:latest".to_string(),
        ]
    }

    async fn invoke(
        &self,
        provider: &dyn ModelProvider,
        model: &str,
        prompt: &str,
        tools: Option<&[ToolSpec]>,
    ) -> Result<GenerateResult, ProviderError> {
        // Llama 3.1 は Ollama 側でチャット形式を処理するため、
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
