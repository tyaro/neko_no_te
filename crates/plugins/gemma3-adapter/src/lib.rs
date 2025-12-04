//! ModelAdapter for Google Gemma 3n (4B Instruct) chat format.
//!
//! Gemma 3 uses `<start_of_turn>user` and `<start_of_turn>model` control tokens.
//! For function calling, Gemma recommends JSON format:
//! `{"name": "function_name", "parameters": {"param": "value"}}`
//!
//! Reference: https://ai.google.dev/gemma/docs/capabilities/function-calling

use async_trait::async_trait;
use model_adapter::{ModelAdapter, ToolSpec};
use model_provider::{GenerateResult, ModelProvider, ProviderError};

/// ModelAdapter for Google Gemma 3n (4B Instruct)
pub struct Gemma3Adapter;

impl Default for Gemma3Adapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Gemma3Adapter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ModelAdapter for Gemma3Adapter {
    fn adapter_name(&self) -> &str {
        "gemma3-adapter"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "gemma3:4b".to_string(),
            "gemma3:latest".to_string(),
            "gemma3".to_string(),
            "gemma3n:e2b".to_string(),
            "gemma3n".to_string(),
        ]
    }

    async fn invoke(
        &self,
        provider: &dyn ModelProvider,
        model: &str,
        prompt: &str,
        tools: Option<&[ToolSpec]>,
    ) -> Result<GenerateResult, ProviderError> {
        // Gemma 3 の関数呼び出しフォーマットに従ってプロンプトを構築
        // システムロールはサポートされないため、ユーザープロンプト内に関数定義を含める
        let mut actual_prompt = prompt.to_string();
        if let Some(t) = tools {
            if !t.is_empty() {
                // JSON 形式で関数定義を構築（Gemma 公式推奨形式）
                let tools_json: Vec<serde_json::Value> = t
                    .iter()
                    .map(|ts| {
                        let mut tool_def = serde_json::json!({
                            "name": ts.name,
                            "description": ts.description.as_deref().unwrap_or(""),
                        });
                        if let Some(schema) = &ts.schema {
                            tool_def["parameters"] = schema.clone();
                        }
                        tool_def
                    })
                    .collect();

                let tools_str = serde_json::to_string_pretty(&tools_json).unwrap_or_default();

                // Gemma 公式ドキュメントに従った関数呼び出しプロンプト
                actual_prompt = format!(
                    "You have access to functions. If you decide to invoke any of the function(s), \
                     you MUST put it in the format of\n\
                     {{\"name\": function name, \"parameters\": dictionary of argument name and its value}}\n\n\
                     You SHOULD NOT include any other text in the response if you call a function\n\
                     {}\n\
                     {}",
                    tools_str, prompt
                );
            }
        }

        provider.generate(model, &actual_prompt).await
    }
}
