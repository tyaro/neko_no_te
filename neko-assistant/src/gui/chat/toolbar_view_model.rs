use super::chat_view::{describe_agent_mode, ChatView};
use chat_core::PromptBuilderRegistry;

#[allow(dead_code)]
pub(super) struct ToolbarViewModel {
    builder_status: String,
    show_mcp_status: bool,
}

impl ToolbarViewModel {
    pub fn from_chat_view(view: &ChatView) -> Self {
        let state = view.chat_state_snapshot();
        Self::from_prompt_state(
            view.prompt_registry.as_ref(),
            &state.active_model,
            view.state.show_mcp_status(),
        )
    }

    #[allow(dead_code)]
    pub fn builder_status(&self) -> &str {
        &self.builder_status
    }

    #[allow(dead_code)]
    pub fn mcp_toggle_label(&self) -> &'static str {
        if self.show_mcp_status {
            "Hide MCP"
        } else {
            "Show MCP"
        }
    }

    pub(super) fn from_prompt_state(
        registry: &PromptBuilderRegistry,
        active_model: &str,
        show_mcp_status: bool,
    ) -> Self {
        let builder_status = describe_prompt_builder(registry, active_model);
        Self {
            builder_status,
            show_mcp_status,
        }
    }
}

fn describe_prompt_builder(registry: &PromptBuilderRegistry, active_model: &str) -> String {
    if registry.is_empty() {
        return "Prompt Builder: 未検出".to_string();
    }

    if let Some(source) = registry.resolve(active_model) {
        let meta = source.metadata();
        let mode = describe_agent_mode(source.preferred_agent());
        if let Some(manifest) = source.manifest() {
            let plugin_name = manifest.name.unwrap_or_else(|| meta.name.clone());
            let location = source
                .plugin_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "(plugin)".into());
            return format!(
                "Prompt Builder: {} v{} ({}, {}) @ {}",
                plugin_name,
                meta.version,
                mode,
                source.origin_label(),
                location
            );
        }
        return format!(
            "Prompt Builder: {} v{} ({}, {})",
            meta.name,
            meta.version,
            mode,
            source.origin_label()
        );
    }

    format!("Prompt Builder: {} 用プラグインなし", active_model)
}

#[cfg(test)]
mod tests {
    use super::ToolbarViewModel;
    use chat_core::{register_builtin_prompt_builders, PromptBuilderRegistry};

    #[test]
    fn reports_available_prompt_builder() {
        let mut registry = PromptBuilderRegistry::from_plugins(&[]);
        register_builtin_prompt_builders(&mut registry);

        let model = ToolbarViewModel::from_prompt_state(&registry, "qwen3:4b-instruct", false);
        assert!(model
            .builder_status()
            .contains("Prompt Builder: Builtin Qwen Prompt"));
        assert_eq!(model.mcp_toggle_label(), "Show MCP");
    }

    #[test]
    fn reports_missing_prompt_builder() {
        let registry = PromptBuilderRegistry::from_plugins(&[]);
        let model = ToolbarViewModel::from_prompt_state(&registry, "unknown-model", true);
        assert_eq!(model.builder_status(), "Prompt Builder: 未検出");
        assert_eq!(model.mcp_toggle_label(), "Hide MCP");
    }

    #[test]
    fn reports_gap_when_registry_has_entries() {
        let mut registry = PromptBuilderRegistry::from_plugins(&[]);
        register_builtin_prompt_builders(&mut registry);

        let model = ToolbarViewModel::from_prompt_state(&registry, "missing-model", true);
        assert_eq!(
            model.builder_status(),
            "Prompt Builder: missing-model 用プラグインなし"
        );
        assert_eq!(model.mcp_toggle_label(), "Hide MCP");
    }
}
