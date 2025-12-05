use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use chat_history::{Conversation, ConversationMetadata, Message, MessageRole};
use ollama_client::{OllamaClient, OllamaListedModel};
use tokio::sync::{mpsc, watch};

use crate::{
    console_log::ConsoleLogRecord, ConversationService, McpManager, McpServerConfig,
    MessageHandler, PromptBuilderRegistry,
};

const PRIMARY_MODEL_ID: &str = "phi4-mini:3.8b";
const CURATED_MODELS: &[(&str, &str)] = &[
    (PRIMARY_MODEL_ID, "Phi-4 Mini 3.8B"),
    ("qwen3:4b-instruct", "Qwen3 4B"),
    ("pakachan/elyza-llama3-8b:latest", "ELYZA Llama3 8B"),
];

/// コントローラー構成設定
pub struct ChatControllerConfig {
    pub conversation_service: ConversationService,
    pub active_model: String,
    pub use_langchain: bool,
    pub ollama_url: String,
    pub mcp_manager: Option<Arc<McpManager>>,
    pub mcp_configs: Vec<McpServerConfig>,
    pub prompt_registry: Option<Arc<PromptBuilderRegistry>>,
    pub welcome_message: String,
}

#[derive(Clone, Debug, Default)]
pub enum McpServerStatus {
    #[default]
    Unknown,
    Ready,
    Error(String),
}

#[derive(Clone, Debug)]
pub struct McpServerMetadata {
    pub name: String,
    pub status: McpServerStatus,
    pub tool_count: usize,
}

impl McpServerMetadata {
    pub fn unknown(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: McpServerStatus::Unknown,
            tool_count: 0,
        }
    }

    pub fn error(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: McpServerStatus::Error(message.into()),
            tool_count: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct McpToolMetadata {
    pub server_name: String,
    pub tool_name: String,
    pub description: String,
}

/// UI に公開するチャット状態スナップショット
#[derive(Clone, Debug)]
pub struct AvailableModel {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct ChatState {
    pub conversation_id: Option<String>,
    pub active_model: String,
    pub messages: Vec<Message>,
    pub conversations: Vec<ConversationMetadata>,
    pub mcp_servers: Vec<McpServerMetadata>,
    pub mcp_tools: Vec<McpToolMetadata>,
    pub console_logs: Vec<ConsoleLogRecord>,
    pub available_models: Vec<AvailableModel>,
}

/// コントローラーが発火するイベント
#[derive(Clone, Debug)]
pub enum ChatEvent {
    StateChanged,
    ConversationsUpdated,
    ModelChanged,
    McpMetadataUpdated,
    ConsoleLogUpdated,
    ModelsUpdated,
    Error(String),
}

/// UI から発行されるコマンド
#[derive(Clone, Debug)]
pub enum ChatCommand {
    SendUserMessage(String),
    SwitchModel(String),
    CreateConversation,
    SwitchConversation(String),
    DeleteConversation(String),
    RefreshConversations,
    RefreshState,
    RefreshMcpMetadata,
    RefreshModels,
}

/// コントローラー操作時に返しうるエラー
#[derive(Debug, Clone)]
pub struct ControllerError {
    message: String,
}

impl ControllerError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

type ControllerResult<T> = Result<T, ControllerError>;

type EventCallback = Arc<dyn Fn(ChatEvent) + Send + Sync>;

struct ChatControllerInner {
    conversation_service: ConversationService,
    message_handler: Arc<MessageHandler>,
    state: Arc<RwLock<ChatState>>,
    state_tx: watch::Sender<ChatState>,
    callbacks: Mutex<HashMap<usize, EventCallback>>,
    next_callback_id: AtomicUsize,
    welcome_message: String,
    mcp_manager: Option<Arc<McpManager>>,
    mcp_configs: Vec<McpServerConfig>,
    ollama_url: String,
}

impl ChatControllerInner {
    fn publish_state(&self) {
        if let Ok(state) = self.state.read() {
            let _ = self.state_tx.send(state.clone());
        }
    }

    fn refresh_state(&self) -> ControllerResult<ChatState> {
        let mut guard = self
            .state
            .write()
            .map_err(|_| ControllerError::new("State lock poisoned"))?;
        guard.conversation_id = self.conversation_service.current_conversation_id();
        guard.messages = self.conversation_service.current_messages();
        let snapshot = guard.clone();
        drop(guard);
        self.publish_state();
        Ok(snapshot)
    }

    fn refresh_conversation_list(&self) -> ControllerResult<()> {
        let mut state = self
            .state
            .write()
            .map_err(|_| ControllerError::new("State lock poisoned"))?;
        state.conversations = self
            .conversation_service
            .list_conversations()
            .map_err(|e| ControllerError::new(e.to_string()))?;
        drop(state);
        self.publish_state();
        Ok(())
    }

    fn emit_event(&self, event: ChatEvent) {
        let callbacks = {
            let guard = self
                .callbacks
                .lock()
                .expect("Callback lock should never be poisoned");
            guard.clone()
        };
        for callback in callbacks.values() {
            callback(event.clone());
        }
    }

    fn emit_error(&self, message: impl Into<String>) {
        self.emit_event(ChatEvent::Error(message.into()));
    }

    fn append_console_log(&self, record: ConsoleLogRecord) {
        let mut should_emit = true;
        if let Ok(mut guard) = self.state.write() {
            guard.console_logs.push(record);
            drop(guard);
            self.publish_state();
        } else {
            eprintln!("Failed to record console log due to poisoned state lock");
            should_emit = false;
        }

        if should_emit {
            self.emit_event(ChatEvent::ConsoleLogUpdated);
        }
    }

    fn emit_state_event(&self) -> ControllerResult<()> {
        self.refresh_state()?;
        self.emit_event(ChatEvent::StateChanged);
        Ok(())
    }

    fn emit_conversation_list(&self) -> ControllerResult<()> {
        self.refresh_conversation_list()?;
        self.emit_event(ChatEvent::ConversationsUpdated);
        Ok(())
    }

    fn refresh_mcp_metadata(self: &Arc<Self>) -> ControllerResult<()> {
        if self.mcp_configs.is_empty() {
            let mut guard = self
                .state
                .write()
                .map_err(|_| ControllerError::new("State lock poisoned"))?;
            guard.mcp_servers.clear();
            guard.mcp_tools.clear();
            drop(guard);
            self.publish_state();
            self.emit_event(ChatEvent::McpMetadataUpdated);
            return Ok(());
        }

        let manager = match &self.mcp_manager {
            Some(manager) => Arc::clone(manager),
            None => {
                let mut guard = self
                    .state
                    .write()
                    .map_err(|_| ControllerError::new("State lock poisoned"))?;
                guard.mcp_servers = self
                    .mcp_configs
                    .iter()
                    .map(|cfg| McpServerMetadata::unknown(cfg.name.clone()))
                    .collect();
                guard.mcp_tools.clear();
                drop(guard);
                self.publish_state();
                self.emit_event(ChatEvent::McpMetadataUpdated);
                return Ok(());
            }
        };

        let state = Arc::clone(&self.state);
        let configs = self.mcp_configs.clone();
        let controller = Arc::clone(self);

        tokio::spawn(async move {
            match manager.get_all_tools().await {
                Ok(tools) => {
                    let mut server_map: HashMap<String, McpServerMetadata> = configs
                        .iter()
                        .map(|cfg| {
                            (
                                cfg.name.clone(),
                                McpServerMetadata::unknown(cfg.name.clone()),
                            )
                        })
                        .collect();

                    let mut tool_metadata = Vec::new();
                    for (server_name, tool) in tools {
                        let entry = server_map
                            .entry(server_name.clone())
                            .or_insert_with(|| McpServerMetadata::unknown(server_name.clone()));
                        entry.status = McpServerStatus::Ready;
                        entry.tool_count += 1;
                        tool_metadata.push(McpToolMetadata {
                            server_name,
                            tool_name: tool.name,
                            description: tool.description,
                        });
                    }

                    let mut should_emit = true;
                    if let Ok(mut guard) = state.write() {
                        let mut servers: Vec<_> = server_map.into_values().collect();
                        servers.sort_by(|a, b| a.name.cmp(&b.name));
                        guard.mcp_servers = servers;
                        guard.mcp_tools = tool_metadata;
                        drop(guard);
                        controller.publish_state();
                    } else {
                        eprintln!("Failed to acquire chat state lock for MCP metadata update");
                        should_emit = false;
                    }

                    if should_emit {
                        controller.emit_event(ChatEvent::McpMetadataUpdated);
                    }
                }
                Err(err) => {
                    let mut should_emit = true;
                    if let Ok(mut guard) = state.write() {
                        guard.mcp_servers = configs
                            .iter()
                            .map(|cfg| McpServerMetadata::error(cfg.name.clone(), err.clone()))
                            .collect();
                        guard.mcp_tools.clear();
                        drop(guard);
                        controller.publish_state();
                    } else {
                        eprintln!("Failed to acquire chat state lock for MCP metadata error");
                        should_emit = false;
                    }

                    if should_emit {
                        controller.emit_event(ChatEvent::McpMetadataUpdated);
                        controller.emit_error(format!("Failed to refresh MCP metadata: {}", err));
                    } else {
                        eprintln!("Skipped emitting MCP metadata error due to poisoned state lock");
                    }
                }
            }
        });

        Ok(())
    }

    fn refresh_available_models(self: &Arc<Self>) -> ControllerResult<()> {
        let base_url = self.ollama_url.clone();
        let state = Arc::clone(&self.state);
        let controller = Arc::clone(self);

        tokio::spawn(async move {
            let result = async {
                let client = OllamaClient::new(&base_url)
                    .map_err(|e| format!("Invalid Ollama URL '{}': {}", base_url, e))?;
                client
                    .list_models()
                    .await
                    .map_err(|e| format!("Failed to list Ollama models: {}", e))
            }
            .await;

            match result {
                Ok(models) => {
                    let presets = build_available_models(models);
                    let active_model = state
                        .read()
                        .map(|guard| guard.active_model.clone())
                        .unwrap_or_default();
                    let contains_active = presets.iter().any(|preset| preset.id == active_model);
                    let fallback_model = if contains_active {
                        None
                    } else {
                        presets
                            .iter()
                            .find(|preset| preset.id == PRIMARY_MODEL_ID)
                            .or_else(|| presets.first())
                            .map(|preset| preset.id.clone())
                    };

                    let mut should_emit = true;
                    if let Ok(mut guard) = state.write() {
                        guard.available_models = presets.clone();
                        drop(guard);
                        controller.publish_state();
                    } else {
                        eprintln!("Failed to update available models due to poisoned state lock");
                        should_emit = false;
                    }

                    if should_emit {
                        controller.emit_event(ChatEvent::ModelsUpdated);
                    }

                    if let Some(fallback) = fallback_model {
                        if fallback != active_model {
                            if let Err(err) = controller.switch_model(fallback.clone()) {
                                controller.emit_error(err.message());
                            }
                        }
                    }
                }
                Err(err) => {
                    controller.emit_error(format!("Failed to refresh model list: {}", err));
                }
            }
        });

        Ok(())
    }

    fn add_callback(self: &Arc<Self>, callback: EventCallback) -> ControllerSubscription {
        let id = self.next_callback_id.fetch_add(1, Ordering::Relaxed);
        self.callbacks
            .lock()
            .expect("Callback lock should never be poisoned")
            .insert(id, callback);
        ControllerSubscription {
            inner: Arc::clone(self),
            id,
            active: true,
        }
    }

    fn remove_callback(&self, id: usize) {
        self.callbacks
            .lock()
            .expect("Callback lock should never be poisoned")
            .remove(&id);
    }

    fn create_conversation(&self) -> ControllerResult<()> {
        let mut conversation = Conversation::new("New Chat");
        conversation.add_message(Message::new(
            MessageRole::System,
            self.welcome_message.clone(),
        ));
        self.conversation_service
            .replace_conversation(conversation)
            .map_err(|e| ControllerError::new(e.to_string()))?;
        self.emit_state_event()?;
        self.emit_conversation_list()?;
        Ok(())
    }

    fn switch_conversation(&self, conversation_id: &str) -> ControllerResult<()> {
        self.conversation_service
            .save_current()
            .map_err(|e| ControllerError::new(e.to_string()))?;
        self.conversation_service
            .load_conversation(conversation_id)
            .map_err(|e| ControllerError::new(e.to_string()))?;
        self.emit_state_event()?;
        Ok(())
    }

    fn delete_conversation(&self, conversation_id: &str) -> ControllerResult<()> {
        if self
            .conversation_service
            .current_conversation_id()
            .map(|id| id == conversation_id)
            .unwrap_or(false)
        {
            return Err(ControllerError::new(
                "Cannot delete the currently active conversation",
            ));
        }
        self.conversation_service
            .delete_conversation(conversation_id)
            .map_err(|e| ControllerError::new(e.to_string()))?;
        self.emit_conversation_list()?;
        Ok(())
    }

    fn switch_model(&self, model: String) -> ControllerResult<()> {
        let mut state = self
            .state
            .write()
            .map_err(|_| ControllerError::new("State lock poisoned"))?;
        if state.active_model == model {
            return Ok(());
        }
        self.message_handler
            .set_model(model.clone())
            .map_err(ControllerError::new)?;
        state.active_model = model.clone();
        drop(state);
        self.publish_state();
        self.emit_event(ChatEvent::ModelChanged);
        Ok(())
    }
}

/// UI 側で保持するサブスクリプションハンドル
pub struct ControllerSubscription {
    inner: Arc<ChatControllerInner>,
    id: usize,
    active: bool,
}

impl ControllerSubscription {
    pub fn cancel(mut self) {
        if self.active {
            self.inner.remove_callback(self.id);
            self.active = false;
        }
    }
}

impl Drop for ControllerSubscription {
    fn drop(&mut self) {
        if self.active {
            self.inner.remove_callback(self.id);
            self.active = false;
        }
    }
}

/// コントローラー外部公開ラッパー
pub struct ChatController {
    inner: Arc<ChatControllerInner>,
    state_rx: watch::Receiver<ChatState>,
}

impl Clone for ChatController {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            state_rx: self.state_rx.clone(),
        }
    }
}

impl ChatController {
    pub fn new(config: ChatControllerConfig) -> Self {
        let ChatControllerConfig {
            conversation_service,
            active_model,
            use_langchain,
            ollama_url,
            mcp_manager,
            mcp_configs,
            prompt_registry,
            welcome_message,
        } = config;

        let (ui_tx, ui_rx) = mpsc::unbounded_channel();
        let message_handler = Arc::new(MessageHandler::new(
            conversation_service.clone(),
            ui_tx,
            use_langchain,
            ollama_url.clone(),
            active_model.clone(),
            mcp_manager.clone(),
            prompt_registry.clone(),
        ));
        let handler_for_callback = Arc::clone(&message_handler);

        let conversations = conversation_service
            .list_conversations()
            .unwrap_or_else(|err| {
                eprintln!("Failed to load conversations: {}", err);
                Vec::new()
            });

        let state = ChatState {
            conversation_id: conversation_service.current_conversation_id(),
            active_model: active_model.clone(),
            messages: conversation_service.current_messages(),
            conversations,
            mcp_servers: mcp_configs
                .iter()
                .map(|cfg| McpServerMetadata::unknown(cfg.name.clone()))
                .collect(),
            mcp_tools: Vec::new(),
            console_logs: Vec::new(),
            available_models: curated_model_list(),
        };

        let (state_tx, state_rx) = watch::channel(state.clone());
        let state_store = Arc::new(RwLock::new(state.clone()));

        let inner = Arc::new(ChatControllerInner {
            conversation_service,
            message_handler,
            state: Arc::clone(&state_store),
            state_tx,
            callbacks: Mutex::new(HashMap::new()),
            next_callback_id: AtomicUsize::new(1),
            welcome_message,
            mcp_manager,
            mcp_configs,
            ollama_url,
        });

        let logs_inner = Arc::downgrade(&inner);
        handler_for_callback.set_console_logger(Some(Arc::new(move |record| {
            if let Some(inner) = logs_inner.upgrade() {
                inner.append_console_log(record);
            }
        })));

        if use_langchain {
            let controller_for_refresh = Arc::clone(&inner);
            handler_for_callback.set_mcp_refresh_callback(Some(Arc::new(move || {
                if let Err(err) = controller_for_refresh.refresh_mcp_metadata() {
                    controller_for_refresh.emit_error(err.message());
                }
            })));
        }

        ChatController::spawn_ui_listener(&inner, ui_rx);

        Self { inner, state_rx }
    }

    pub fn subscribe<F>(&self, callback: F) -> ControllerSubscription
    where
        F: Fn(ChatEvent) + Send + Sync + 'static,
    {
        self.inner.add_callback(Arc::new(callback))
    }

    pub fn handle_command(&self, command: ChatCommand) -> ControllerResult<()> {
        match command {
            ChatCommand::SendUserMessage(text) => {
                if text.trim().is_empty() {
                    return Ok(());
                }
                self.inner.message_handler.handle_user_message(text);
                Ok(())
            }
            ChatCommand::SwitchModel(model) => self.inner.switch_model(model),
            ChatCommand::CreateConversation => self.inner.create_conversation(),
            ChatCommand::SwitchConversation(id) => self.inner.switch_conversation(&id),
            ChatCommand::DeleteConversation(id) => self.inner.delete_conversation(&id),
            ChatCommand::RefreshConversations => self.inner.emit_conversation_list(),
            ChatCommand::RefreshState => self.inner.emit_state_event(),
            ChatCommand::RefreshMcpMetadata => self.inner.refresh_mcp_metadata(),
            ChatCommand::RefreshModels => self.inner.refresh_available_models(),
        }
    }

    pub fn state_snapshot(&self) -> ChatState {
        self.state_rx.borrow().clone()
    }

    pub fn state_stream(&self) -> watch::Receiver<ChatState> {
        self.state_rx.clone()
    }

    /// Append a record to the console logs visible in the UI.
    pub fn append_console_log(&self, kind: crate::ConsoleLogKind, content: impl Into<String>) {
        let record = ConsoleLogRecord::new(kind, content.into());
        self.inner.append_console_log(record);
    }

    fn spawn_ui_listener(inner: &Arc<ChatControllerInner>, mut rx: mpsc::UnboundedReceiver<()>) {
        let controller = Arc::clone(inner);
        tokio::spawn(async move {
            while rx.recv().await.is_some() {
                if let Err(err) = controller.emit_state_event() {
                    controller.emit_error(err.message());
                }
            }
        });
    }
}

impl From<&str> for ControllerError {
    fn from(value: &str) -> Self {
        ControllerError::new(value)
    }
}

impl From<String> for ControllerError {
    fn from(value: String) -> Self {
        ControllerError::new(value)
    }
}

fn curated_model_list() -> Vec<AvailableModel> {
    CURATED_MODELS
        .iter()
        .map(|(id, label)| AvailableModel {
            id: (*id).to_string(),
            label: (*label).to_string(),
        })
        .collect()
}

fn build_available_models(models: Vec<OllamaListedModel>) -> Vec<AvailableModel> {
    let mut presets = curated_model_list();
    let mut seen: HashSet<String> = presets.iter().map(|preset| preset.id.clone()).collect();

    for model in models {
        if !seen.insert(model.name.clone()) {
            continue;
        }
        let label = detected_model_label(&model);
        let id = model.name;
        presets.push(AvailableModel { id, label });
    }

    presets
}

fn detected_model_label(model: &OllamaListedModel) -> String {
    if let Some(details) = &model.details {
        if let Some(param) = &details.parameter_size {
            return format!("{} ({})", model.name, param);
        }
        if let Some(family) = &details.family {
            return format!("{} ({})", model.name, family);
        }
    }

    if let Some(size) = model.size {
        let size_mb = size as f64 / (1024.0 * 1024.0);
        return format!("{} ({:.1} MB)", model.name, size_mb);
    }

    model.name.clone()
}
