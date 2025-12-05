use super::chat_view::ChatView;
use super::chat_view_state::ChatViewState;
use super::controller_facade::ChatControllerFacade;
use super::event_loop::ChatEventLoop;
use chat_core::{
    load_mcp_config, ChatCommand, ChatController, ChatControllerConfig, ControllerSubscription,
    ConversationService, McpManager, McpServerConfig, PluginEntry, PromptBuilderRegistry, ConsoleLogKind,
};
use chat_history::{Conversation, ConversationManager, Message, MessageRole};
use gpui::{Context, Window};
use gpui_component::input::InputEvent;
use gpui_component::select::SelectEvent;
use neko_ui::ModelPreset;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

type McpConfigLoader = dyn Fn() -> Result<Vec<McpServerConfig>, String> + Send + Sync;

pub struct ChatViewBuilder {
    repo_root: PathBuf,
    plugins: Vec<PluginEntry>,
    prompt_registry: Arc<PromptBuilderRegistry>,
    config_loader: Arc<McpConfigLoader>,
}

pub struct ChatViewParts {
    pub repo_root: PathBuf,
    pub plugins: Vec<PluginEntry>,
    pub prompt_registry: Arc<PromptBuilderRegistry>,
    pub controller: ChatControllerFacade,
    pub event_loop: ChatEventLoop,
    pub state: ChatViewState,
}

impl ChatViewBuilder {
    pub fn new(
        repo_root: PathBuf,
        plugins: Vec<PluginEntry>,
        prompt_registry: Arc<PromptBuilderRegistry>,
    ) -> Self {
        Self {
            repo_root,
            plugins,
            prompt_registry,
            config_loader: Arc::new(load_mcp_config),
        }
    }

    pub fn build(self, window: &mut Window, cx: &mut Context<ChatView>) -> ChatViewParts {
        let ChatViewBuilder {
            repo_root,
            plugins,
            prompt_registry,
            config_loader,
        } = self;

        let config = app_config::AppConfig::load_or_default();
        let use_langchain = config.use_langchain;
        let active_model = config.default_model.clone();

        let (conversation_service, welcome_message) =
            Self::initialize_conversation_context(use_langchain);
        let (mcp_manager, mcp_configs) =
            Self::initialize_mcp_context_inner(&config_loader, use_langchain);

        let (controller, event_loop, subscription) = Self::initialize_controller(
            &config,
            active_model,
            use_langchain,
            mcp_manager,
            mcp_configs.clone(),
            conversation_service,
            prompt_registry.clone(),
            welcome_message,
        );

        let mut state = ChatViewState::new(window, cx, &controller.state_snapshot(), &repo_root);
        Self::bind_input_listeners(cx, window, &mut state, controller.clone());

        let facade = ChatControllerFacade::new(controller.clone(), subscription);
        Self::run_initial_commands(&facade, use_langchain, &mcp_configs);

        ChatViewParts {
            repo_root,
            plugins,
            prompt_registry,
            controller: facade,
            event_loop,
            state,
        }
    }

    fn initialize_conversation_context(use_langchain: bool) -> (ConversationService, String) {
        let storage_dir = ConversationManager::default_storage_dir().unwrap_or_else(|err| {
            eprintln!("Failed to determine conversation storage dir: {}", err);
            env::temp_dir().join("neko-assistant").join("conversations")
        });
        let conversation_manager = match ConversationManager::new(&storage_dir) {
            Ok(manager) => Arc::new(Mutex::new(manager)),
            Err(err) => {
                eprintln!(
                    "Failed to initialize ConversationManager at {:?}: {}",
                    storage_dir, err
                );
                let fallback_dir = env::temp_dir()
                    .join("neko-assistant")
                    .join("fallback_conversations");
                let fallback_manager =
                    ConversationManager::new(&fallback_dir).unwrap_or_else(|create_err| {
                        panic!(
                            "Failed to initialize fallback ConversationManager: {}",
                            create_err
                        )
                    });
                Arc::new(Mutex::new(fallback_manager))
            }
        };

        let mut conversation = Conversation::new("New Chat");
        let welcome_message = if use_langchain {
            "Welcome to Neko Assistant (LangChain mode enabled)".to_string()
        } else {
            "Welcome to Neko Assistant".to_string()
        };
        conversation.add_message(Message::new(MessageRole::System, welcome_message.clone()));
        let conversation_arc = Arc::new(Mutex::new(conversation));
        let conversation_service =
            ConversationService::new(conversation_arc, conversation_manager.clone());

        (conversation_service, welcome_message)
    }

    #[cfg(test)]
    fn initialize_mcp_context(
        &self,
        use_langchain: bool,
    ) -> (Option<Arc<McpManager>>, Vec<McpServerConfig>) {
        Self::initialize_mcp_context_inner(&self.config_loader, use_langchain)
    }

    fn initialize_mcp_context_inner(
        loader: &Arc<McpConfigLoader>,
        use_langchain: bool,
    ) -> (Option<Arc<McpManager>>, Vec<McpServerConfig>) {
        if !use_langchain {
            return (None, Vec::new());
        }

        match loader() {
            Ok(configs) if !configs.is_empty() => {
                let manager = Arc::new(McpManager::new(configs.clone()));
                (Some(manager), configs)
            }
            Ok(_) => (None, Vec::new()),
            Err(e) => {
                eprintln!("Failed to load MCP config: {}", e);
                (None, Vec::new())
            }
        }
    }

    #[cfg(test)]
    pub fn with_config_loader(mut self, loader: Arc<McpConfigLoader>) -> Self {
        self.config_loader = loader;
        self
    }

    #[allow(clippy::too_many_arguments)]
    fn initialize_controller(
        config: &app_config::AppConfig,
        active_model: String,
        use_langchain: bool,
        mcp_manager: Option<Arc<McpManager>>,
        mcp_configs: Vec<McpServerConfig>,
        conversation_service: ConversationService,
        prompt_registry: Arc<PromptBuilderRegistry>,
        welcome_message: String,
    ) -> (Arc<ChatController>, ChatEventLoop, ControllerSubscription) {
        let controller = Arc::new(ChatController::new(ChatControllerConfig {
            conversation_service,
            active_model,
            use_langchain,
            ollama_url: config.ollama_base_url.clone(),
            mcp_manager,
            mcp_configs: mcp_configs.clone(),
            prompt_registry: Some(prompt_registry),
            welcome_message,
        }));

        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let event_loop = ChatEventLoop::new(Arc::new(Mutex::new(event_rx)));
        let subscription = controller.subscribe(move |event| {
            if event_tx.send(event).is_err() {
                eprintln!("Failed to deliver chat event to UI");
            }
        });

        (controller, event_loop, subscription)
    }

    fn bind_input_listeners(
        cx: &mut Context<ChatView>,
        window: &mut Window,
        state: &mut ChatViewState,
        controller: Arc<ChatController>,
    ) {
        let input_state = state.input_state().clone();
        // model selector no longer exposes a free-text input; selection-only.
        let model_select_state = state.model_selector().select_state().clone();

        let handler_sub = controller.clone();
        let mut subs = vec![cx.subscribe_in(
            &input_state,
            window,
            move |_this, field, ev: &InputEvent, window, cx| {
                if let InputEvent::PressEnter { secondary } = ev {
                    if !secondary {
                        let val = field.read(cx).value();
                        let trimmed = val.trim();
                        if trimmed.is_empty() {
                            return;
                        }

                        let user_input = trimmed.to_string();
                        if let Err(err) = handler_sub
                            .handle_command(ChatCommand::SendUserMessage(user_input.clone()))
                        {
                            eprintln!("Failed to send message: {}", err.message());
                        }

                        field.update(cx, |view, cx| view.set_value("", window, cx));
                    }
                }
            },
        )];

        // No text-input subscription for model selector — use the selection widget only.

        let select_state_for_events = model_select_state.clone();
        subs.push(cx.subscribe_in(
            &select_state_for_events,
            window,
            move |this, _state, event: &SelectEvent<Vec<ModelPreset>>, window, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    // Normalize (strip trailing parenthetical label) — e.g. "gemma3n:e2b (4.5B)" -> "gemma3n:e2b"
                    let mut normalized = value.trim().to_string();
                    if let Some(idx) = normalized.find('(') {
                        normalized = normalized[..idx].trim().to_string();
                    }

                    // If we have plugin metadata, ensure a plugin advertises support for this model id
                    let plugin_match = this
                        .plugins
                        .iter()
                        .any(|p| p.metadata.as_ref().map(|m| m.models.iter().any(|mid| mid == &normalized)).unwrap_or(false));

                    if !plugin_match {
                        // No adapter plugin explicitly lists this model — warn and do not switch.
                        // (This avoids selecting a UI label that doesn't map to an installed adapter.)
                        this.controller
                            .controller()
                            .append_console_log(
                                ConsoleLogKind::Error,
                                format!(
                                    "No installed adapter plugin matches requested model: {}",
                                    normalized
                                ),
                            );
                        return;
                    }

                    // Log that we found a matching installed adapter and are switching
                    this.controller
                        .controller()
                        .append_console_log(
                            ConsoleLogKind::Output,
                            format!("Plugin match found for model '{}', switching..", normalized),
                        );

                    if let Err(err) = this.state.model_selector().switch_model(
                        &this.controller,
                        &normalized,
                        window,
                        cx,
                    ) {
                        eprintln!("Failed to switch model: {}", err);
                    }
                }
            },
        ));

        state.set_subscriptions(subs);
    }

    fn run_initial_commands(
        controller: &ChatControllerFacade,
        use_langchain: bool,
        mcp_configs: &[McpServerConfig],
    ) {
        if let Err(err) = controller.handle_command(ChatCommand::RefreshConversations) {
            eprintln!("Failed to refresh conversations: {}", err.message());
        }

        if use_langchain && !mcp_configs.is_empty() {
            if let Err(err) = controller.handle_command(ChatCommand::RefreshMcpMetadata) {
                eprintln!("Failed to refresh MCP metadata: {}", err.message());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builder_stub() -> ChatViewBuilder {
        ChatViewBuilder::new(
            PathBuf::new(),
            Vec::new(),
            Arc::new(PromptBuilderRegistry::from_plugins(&[])),
        )
    }

    #[test]
    fn mcp_context_disabled_without_langchain() {
        let builder = builder_stub();
        let (manager, configs) = builder.initialize_mcp_context(false);
        assert!(manager.is_none());
        assert!(configs.is_empty());
    }

    #[test]
    fn mcp_context_falls_back_on_loader_error() {
        let loader: Arc<McpConfigLoader> =
            Arc::new(|| -> Result<Vec<McpServerConfig>, String> { Err("boom".into()) });
        let builder = builder_stub().with_config_loader(loader);
        let (manager, configs) = builder.initialize_mcp_context(true);
        assert!(manager.is_none());
        assert!(configs.is_empty());
    }
}
