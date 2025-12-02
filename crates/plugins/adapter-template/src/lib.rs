//! Adapter template: copy this crate and implement the TODO sections.
//!
//! Steps:
//!  - copy this folder to `crates/plugins/my-adapter`
//!  - update `Cargo.toml` (name, description, authors)
//!  - implement the adapter logic below

use async_trait::async_trait;
use model_adapter::{ModelAdapter, ToolSpec};
use model_provider::{GenerateResult, ModelProvider, ProviderError};

/// Example adapter struct — rename to your adapter name.
pub struct MyAdapter;

impl MyAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ModelAdapter for MyAdapter {
    fn adapter_name(&self) -> &str {
        "my-adapter"
    }

    fn supported_models(&self) -> Vec<String> {
        // Return model names this adapter supports, e.g. ["qwen3:8b".into()]
        vec!["my-model:latest".to_string()]
    }

    async fn invoke(
        &self,
        provider: &dyn ModelProvider,
        model: &str,
        prompt: &str,
        tools: Option<&[ToolSpec]>,
    ) -> Result<GenerateResult, ProviderError> {
        // TODO: Implement model-specific payload formatting.
        // Example strategies:
        // - Some models expect `messages` (chat format) instead of raw `prompt`.
        // - Function-calling schemas might require embedding `tools` as JSON
        //   under a specific key or as part of the prompt.

        // Simple default: append a short tool description to the prompt.
        let mut actual_prompt = prompt.to_string();
        if let Some(t) = tools {
            if !t.is_empty() {
                let desc = serde_json::to_string(&t).unwrap_or_default();
                actual_prompt.push_str("\n\nTools: ");
                actual_prompt.push_str(&desc);
            }
        }

        // Delegate to provider. Customize if provider requires specific API.
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
    async fn template_invokes_provider() {
        let a = MyAdapter::new();
        let p = DummyProvider;
        let res = a.invoke(&p, "my-model:latest", "hi", None).await.unwrap();
        assert!(res.text.contains("hi"));
    }
}
