use super::chat_view::ChatView;
use chat_core::{ChatController, PluginEntry};
use gpui::SharedString;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub(super) struct MenuContext {
    controller: Arc<ChatController>,
    repo_root: Arc<PathBuf>,
    plugins: Arc<Vec<PluginEntry>>,
    show_mcp_status: bool,
}

impl MenuContext {
    pub fn from_chat_view(view: &ChatView) -> Self {
        Self {
            controller: view.controller.controller(),
            repo_root: Arc::new(view.repo_root.clone()),
            plugins: Arc::new(view.plugins.clone()),
            show_mcp_status: view.state.show_mcp_status(),
        }
    }

    pub fn controller(&self) -> Arc<ChatController> {
        Arc::clone(&self.controller)
    }

    pub fn repo_root(&self) -> Arc<PathBuf> {
        Arc::clone(&self.repo_root)
    }

    pub fn plugins(&self) -> Arc<Vec<PluginEntry>> {
        Arc::clone(&self.plugins)
    }

    pub fn mcp_toggle_label(&self) -> SharedString {
        if self.show_mcp_status() {
            SharedString::from("Hide MCP Status")
        } else {
            SharedString::from("Show MCP Status")
        }
    }

    pub fn show_mcp_status(&self) -> bool {
        self.show_mcp_status
    }

    #[cfg(test)]
    pub fn new_for_testing(
        controller: Arc<ChatController>,
        repo_root: PathBuf,
        plugins: Vec<PluginEntry>,
        show_mcp_status: bool,
    ) -> Self {
        Self {
            controller,
            repo_root: Arc::new(repo_root),
            plugins: Arc::new(plugins),
            show_mcp_status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MenuContext;
    use chat_core::{
        ChatController, ChatControllerConfig, ConversationService, PluginEntry,
        PromptBuilderRegistry,
    };
    use chat_history::{Conversation, ConversationManager};
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    fn build_controller() -> Arc<ChatController> {
        let conversation = Arc::new(Mutex::new(Conversation::new("Test")));
        let temp_dir = tempdir().unwrap();
        let manager = Arc::new(Mutex::new(
            ConversationManager::new(temp_dir.path()).unwrap(),
        ));
        let service = ConversationService::new(conversation, manager);
        Arc::new(ChatController::new(ChatControllerConfig {
            conversation_service: service,
            active_model: "phi4-mini:3.8b".into(),
            use_langchain: false,
            ollama_url: "http://localhost:11434".into(),
            mcp_manager: None,
            mcp_configs: Vec::new(),
            prompt_registry: Some(Arc::new(PromptBuilderRegistry::from_plugins(&[]))),
            welcome_message: "hi".into(),
        }))
    }

    #[tokio::test]
    async fn menu_context_reuses_handles_and_labels() {
        let controller = build_controller();
        let repo_root = std::path::PathBuf::from("C:/repo");
        let plugins = vec![PluginEntry {
            dir_name: "sample".into(),
            path: std::path::PathBuf::from("C:/repo/plugins/sample"),
            enabled: true,
            metadata: None,
        }];

        let context = MenuContext::new_for_testing(controller, repo_root, plugins, true);

        let repo_a = context.repo_root();
        let repo_b = context.repo_root();
        assert!(Arc::ptr_eq(&repo_a, &repo_b));

        let plugins_a = context.plugins();
        let plugins_b = context.plugins();
        assert!(Arc::ptr_eq(&plugins_a, &plugins_b));

        assert_eq!(context.mcp_toggle_label().as_ref(), "Hide MCP Status");
        assert!(context.show_mcp_status());
    }

    #[tokio::test]
    async fn menu_context_toggle_label_matches_state() {
        let controller = build_controller();
        let repo_root = std::path::PathBuf::from("C:/repo");
        let plugins = vec![];

        let hide_context = MenuContext::new_for_testing(
            controller.clone(),
            repo_root.clone(),
            plugins.clone(),
            true,
        );
        assert_eq!(hide_context.mcp_toggle_label().as_ref(), "Hide MCP Status");
        assert!(hide_context.show_mcp_status());

        let show_context = MenuContext::new_for_testing(controller, repo_root, plugins, false);
        assert_eq!(show_context.mcp_toggle_label().as_ref(), "Show MCP Status");
        assert!(!show_context.show_mcp_status());
    }
}
