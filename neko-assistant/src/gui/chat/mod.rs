mod conversation_actions;
mod sidebar;
mod toolbar;

use super::mcp_manager;
use crate::conversation_service::ConversationService;
use crate::message_handler::MessageHandler;
use crate::plugins::{PluginEntry, PromptBuilderRegistry};
use crate::prompt_builders;
use chat_history::{Conversation, Message, MessageRole};
use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::select::{SelectEvent, SelectState};
use gpui_component::Root;
use gpui_component::StyledExt;
use neko_ui::{
    model_selector::{self, ModelPreset},
    ChatBubble, MessageType,
};
use ollama_client::{OllamaClient, OllamaListedModel};
use prompt_spi::PromptAgentMode;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use ui_utils::ScrollManager;

use conversation_actions::ConversationActions;

trait OverflowScrollExt: Sized {
    fn overflow_y_scroll(self) -> Self;
}

impl OverflowScrollExt for Div {
    fn overflow_y_scroll(mut self) -> Self {
        self.style().overflow.y = Some(Overflow::Scroll);
        self
    }
}

const PRIMARY_MODEL_ID: &str = "phi4-mini:3.8b";
const CURATED_MODELS: &[(&str, &str)] = &[
    (PRIMARY_MODEL_ID, "Phi-4 Mini 3.8B"),
    ("qwen3:4b-instruct", "Qwen3 4B"),
    ("pakachan/elyza-llama3-8b:latest", "ELYZA Llama3 8B"),
];

// Chat view with proper chat bubbles
pub struct ChatView {
    repo_root: PathBuf,
    plugins: Vec<PluginEntry>,
    prompt_registry: Arc<PromptBuilderRegistry>,
    active_model: String,
    model_presets: Vec<ModelPreset>,
    model_select_state: gpui::Entity<SelectState<Vec<ModelPreset>>>,
    model_selector_input: gpui::Entity<InputState>,
    editor_input: gpui::Entity<InputState>,
    input_state: gpui::Entity<InputState>,
    conversation_service: ConversationService,
    conversations_list: Vec<chat_history::ConversationMetadata>,
    conversation_actions: ConversationActions,
    ui_update_rx: Arc<Mutex<mpsc::UnboundedReceiver<()>>>,
    model_updates_rx: Arc<Mutex<mpsc::UnboundedReceiver<Vec<ModelPreset>>>>,
    scroll_manager: ScrollManager,
    _message_handler: Arc<MessageHandler>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl ChatView {
    pub fn new(
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
        repo_root: PathBuf,
        plugins: Vec<PluginEntry>,
        prompt_registry: Arc<PromptBuilderRegistry>,
    ) -> Self {
        // 設定を読み込み
        let config = app_config::AppConfig::load_or_default();
        let use_langchain = config.use_langchain;
        let active_model = config.default_model.clone();

        // ConversationManagerを初期化
        let storage_dir = chat_history::ConversationManager::default_storage_dir()
            .expect("Failed to get storage directory");
        let manager = chat_history::ConversationManager::new(storage_dir)
            .expect("Failed to initialize ConversationManager");
        let conversation_manager = Arc::new(Mutex::new(manager));

        // 会話を初期化
        let mut conversation = Conversation::new("New Chat");
        let welcome_msg = if use_langchain {
            Message::new(
                MessageRole::System,
                "Welcome to Neko Assistant (LangChain mode enabled)".to_string(),
            )
        } else {
            Message::new(MessageRole::System, "Welcome to Neko Assistant".to_string())
        };
        conversation.add_message(welcome_msg);
        let conversation_arc = Arc::new(Mutex::new(conversation));
        let conversation_service =
            ConversationService::new(conversation_arc.clone(), conversation_manager.clone());

        // UI更新通知用チャネル
        let (ui_tx, ui_rx) = mpsc::unbounded_channel();
        let ui_update_rx = Arc::new(Mutex::new(ui_rx));

        // MCP Managerを初期化（LangChainモード時のみ）
        let mcp_manager = if use_langchain {
            match crate::mcp_client::load_mcp_config() {
                Ok(configs) if !configs.is_empty() => {
                    Some(Arc::new(crate::mcp_manager::McpManager::new(configs)))
                }
                Ok(_) => {
                    println!("No MCP servers configured");
                    None
                }
                Err(e) => {
                    eprintln!("Failed to load MCP config: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // MessageHandlerを初期化
        let message_handler = Arc::new(MessageHandler::new(
            conversation_service.clone(),
            ui_tx.clone(),
            use_langchain,
            config.ollama_base_url.clone(),
            active_model.clone(),
            mcp_manager,
            Some(prompt_registry.clone()),
        ));

        // ConversationActionsを初期化
        let conversation_actions = ConversationActions::new(conversation_service.clone());

        // create an InputState entity with multi-line support
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Type your message... (Enter to send)")
                .auto_grow(3, 10) // 3〜10行の自動成長
        });

        let model_selector_input = cx.new(|cx| {
            let mut state =
                InputState::new(window, cx).placeholder("モデルID (例: phi4-mini:3.8b)");
            state.set_value(&active_model, window, cx);
            state
        });

        let model_presets = curated_model_presets();
        let select_items = model_presets.clone();
        let selected_value = if select_items.iter().any(|preset| preset.id == active_model) {
            Some(active_model.clone())
        } else {
            None
        };
        let model_select_state = cx.new(|cx| {
            let mut state = SelectState::new(select_items.clone(), None, window, cx);
            if let Some(value) = selected_value.clone() {
                state.set_selected_value(&value, window, cx);
            }
            state
        });
        let editor_input = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("markdown")
                .auto_grow(6, 16)
                .placeholder("Notes / prompt scratchpad...")
        });
        let (model_tx, model_rx) = mpsc::unbounded_channel();
        let model_updates_rx = Arc::new(Mutex::new(model_rx));

        // Subscribe to input events
        let handler_sub = message_handler.clone();
        let mut subs = vec![cx.subscribe_in(
            &input_state,
            window,
            move |_this, state, ev: &InputEvent, window, cx| {
                if let InputEvent::PressEnter { secondary } = ev {
                    // Enterで送信、Shift+Enterで改行
                    if !secondary {
                        let val = state.read(cx).value();
                        let trimmed = val.trim();
                        if trimmed.is_empty() {
                            return;
                        }

                        let user_input = trimmed.to_string();

                        // MessageHandlerに処理を委譲
                        handler_sub.handle_user_message(user_input);

                        // Clear input
                        let _ = state.update(cx, |view, cx| view.set_value("", window, cx));
                    }
                }
            },
        )];

        subs.push(cx.subscribe_in(
            &model_selector_input,
            window,
            move |this, state, ev: &InputEvent, window, cx| {
                if let InputEvent::PressEnter { secondary } = ev {
                    if *secondary {
                        return;
                    }
                    let value = state.read(cx).value().trim().to_string();
                    if value.is_empty() {
                        return;
                    }
                    this.switch_model(&value, window, cx);
                }
            },
        ));

        let select_state_for_events = model_select_state.clone();
        subs.push(cx.subscribe_in(
            &select_state_for_events,
            window,
            move |this, _state, event: &SelectEvent<Vec<ModelPreset>>, window, cx| {
                if let SelectEvent::Confirm(Some(value)) = event {
                    this.switch_model(value, window, cx);
                }
            },
        ));

        let view = Self {
            repo_root,
            plugins,
            prompt_registry,
            active_model,
            model_presets,
            model_select_state,
            model_selector_input,
            editor_input,
            input_state,
            conversation_service,
            conversations_list: Vec::new(), // 初期化時は空、render時に読み込む
            conversation_actions,
            ui_update_rx,
            model_updates_rx,
            scroll_manager: ScrollManager::new(),
            _message_handler: message_handler,
            _subscriptions: subs,
        };

        Self::start_model_discovery(config.ollama_base_url.clone(), model_tx);

        view
    }

    /// 新規会話を作成して切り替え
    fn create_new_conversation(
        &mut self,
        _: &ClickEvent,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        if let Err(e) = self.conversation_actions.create_new_conversation(
            &self.input_state,
            &mut self.scroll_manager,
            window,
            cx,
        ) {
            eprintln!("Failed to create new conversation: {}", e);
        }

        // 会話リストをリフレッシュ（次回renderで更新）
        self.conversations_list.clear();

        // UI更新
        cx.notify();
    }

    /// 指定IDの会話に切り替え
    fn switch_conversation(
        &mut self,
        conversation_id: &str,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        if let Err(e) = self.conversation_actions.switch_conversation(
            conversation_id,
            &self.input_state,
            &mut self.scroll_manager,
            window,
            cx,
        ) {
            eprintln!("Failed to switch conversation: {}", e);
        }

        // UI更新
        cx.notify();
    }

    /// 指定IDの会話を削除
    fn delete_conversation(&mut self, conversation_id: &str, cx: &mut gpui::Context<Self>) {
        if let Err(e) = self
            .conversation_actions
            .delete_conversation(conversation_id)
        {
            eprintln!("Failed to delete conversation: {}", e);
            return;
        }

        // 会話リストをリフレッシュ
        self.conversations_list.clear();

        // UI更新
        cx.notify();
    }

    fn switch_model(
        &mut self,
        model_id: &str,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let normalized = model_id.trim();
        if normalized.is_empty() {
            return;
        }

        if self.active_model == normalized {
            let _ = self.model_selector_input.update(cx, |state, cx| {
                state.set_value(&self.active_model, window, cx);
            });
            return;
        }

        self.active_model = normalized.to_string();
        let _ = self.model_selector_input.update(cx, |state, cx| {
            state.set_value(&self.active_model, window, cx);
        });
        if let Err(err) = self._message_handler.set_model(self.active_model.clone()) {
            eprintln!("Failed to update active model: {}", err);
        }
        if let Err(err) = self.persist_model_selection() {
            eprintln!("Failed to persist model selection: {}", err);
        }
        self.sync_model_selector_selection(window, cx);
        cx.notify();
    }

    fn persist_model_selection(&self) -> Result<(), String> {
        let mut config = app_config::AppConfig::load_or_default();
        config.default_model = self.active_model.clone();
        config.save().map_err(|e| e.to_string())
    }

    fn render_model_selector_row(&self) -> impl IntoElement {
        div()
            .border_t_1()
            .border_color(rgb(0x333333))
            .bg(rgb(0x101010))
            .p_2()
            .v_flex()
            .gap_1()
            .child(div().text_sm().text_color(rgb(0xaaaaaa)).child("Model"))
            .child(model_selector::model_selector(
                &self.model_select_state,
                &self.model_selector_input,
            ))
    }

    fn render_editor_console(&mut self) -> Div {
        let logs = self.conversation_service.current_messages();

        let console_body: AnyElement = if logs.is_empty() {
            div()
                .flex_1()
                .justify_center()
                .items_center()
                .text_sm()
                .text_color(rgb(0x777777))
                .child("No messages yet")
                .into_any_element()
        } else {
            let items = logs.into_iter().map(|msg| {
                let role = format_log_role(msg.role);
                div()
                    .text_xs()
                    .text_color(rgb(0xcccccc))
                    .child(format!("[{}] {}", role, msg.content))
            });

            let scroll_body = div().v_flex().gap_1().children(items);

            div()
                .flex_1()
                .overflow_hidden()
                .child(div().size_full().overflow_y_scroll().child(scroll_body))
                .into_any_element()
        };

        div()
            .w(px(320.0))
            .h_full()
            .border_r_1()
            .border_color(rgb(0x333333))
            .v_flex()
            .child(
                div()
                    .p_2()
                    .border_b_1()
                    .border_color(rgb(0x333333))
                    .v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xaaaaaa))
                            .child("Scratchpad"),
                    )
                    .child(
                        Input::new(&self.editor_input)
                            .w_full()
                            .h(px(200.0))
                            .text_sm(),
                    ),
            )
            .child(
                div()
                    .p_2()
                    .flex_1()
                    .v_flex()
                    .gap_1()
                    .child(div().text_sm().text_color(rgb(0xaaaaaa)).child("Console"))
                    .child(console_body),
            )
    }

    fn sync_model_selector_items(&self, window: &mut gpui::Window, cx: &mut gpui::Context<Self>) {
        let presets = self.model_presets.clone();
        let active = self.active_model.clone();
        let has_match = presets.iter().any(|preset| preset.id == active);
        let _ = self.model_select_state.update(cx, |state, cx| {
            state.set_items(presets, window, cx);
            if has_match {
                state.set_selected_value(&active, window, cx);
            } else {
                state.set_selected_index(None, window, cx);
            }
        });
    }

    fn sync_model_selector_selection(
        &self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let active = self.active_model.clone();
        let has_match = self.model_presets.iter().any(|preset| preset.id == active);
        let _ = self.model_select_state.update(cx, |state, cx| {
            if has_match {
                state.set_selected_value(&active, window, cx);
            } else {
                state.set_selected_index(None, window, cx);
            }
        });
    }

    fn apply_detected_models(
        &mut self,
        presets: Vec<ModelPreset>,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let contains_active = presets.iter().any(|preset| preset.id == self.active_model);
        self.model_presets = presets;
        self.sync_model_selector_items(window, cx);

        if !contains_active {
            if self
                .model_presets
                .iter()
                .any(|preset| preset.id == PRIMARY_MODEL_ID)
            {
                self.switch_model(PRIMARY_MODEL_ID, window, cx);
            } else if let Some(first) = self.model_presets.first() {
                let next_model = first.id.clone();
                self.switch_model(&next_model, window, cx);
            }
        } else {
            cx.notify();
        }
    }

    fn start_model_discovery(base_url: String, tx: mpsc::UnboundedSender<Vec<ModelPreset>>) {
        thread::spawn(move || {
            let runtime = match Runtime::new() {
                Ok(rt) => rt,
                Err(err) => {
                    eprintln!(
                        "Failed to initialize Tokio runtime for model discovery: {}",
                        err
                    );
                    return;
                }
            };

            let result = runtime.block_on(async {
                let client = OllamaClient::new(&base_url)
                    .map_err(|e| format!("Invalid Ollama URL '{}': {}", base_url, e))?;
                client
                    .list_models()
                    .await
                    .map_err(|e| format!("Failed to list Ollama models: {}", e))
            });

            match result {
                Ok(models) => {
                    let presets = build_model_presets(models);
                    let _ = tx.send(presets);
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        });
    }
}

fn curated_model_presets() -> Vec<ModelPreset> {
    CURATED_MODELS
        .iter()
        .map(|(id, label)| ModelPreset::new(*id, *label))
        .collect()
}

fn build_model_presets(models: Vec<OllamaListedModel>) -> Vec<ModelPreset> {
    let mut presets = curated_model_presets();
    let mut seen: HashSet<String> = presets.iter().map(|preset| preset.id.clone()).collect();

    for model in models {
        if !seen.insert(model.name.clone()) {
            continue;
        }
        let label = detected_model_label(&model);
        presets.push(ModelPreset::new(model.name, label));
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

fn format_log_role(role: MessageRole) -> &'static str {
    match role {
        MessageRole::User => "User",
        MessageRole::Assistant => "Assistant",
        MessageRole::System => "System",
        MessageRole::Error => "Error",
    }
}

impl gpui::Render for ChatView {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        gpui_component::init(cx);

        // 会話一覧を更新（初回またはUI更新時）
        if self.conversations_list.is_empty() {
            if let Ok(list) = self.conversation_actions.list_conversations() {
                self.conversations_list = list;
            }
        }

        // チャネルから更新通知をチェック
        if let Ok(mut rx) = self.ui_update_rx.try_lock() {
            while rx.try_recv().is_ok() {
                self.scroll_manager.mark_scroll_to_bottom();
                cx.notify();
            }
        }

        let mut pending_model_updates = Vec::new();
        if let Ok(mut rx) = self.model_updates_rx.try_lock() {
            while let Ok(presets) = rx.try_recv() {
                pending_model_updates.push(presets);
            }
        }
        for presets in pending_model_updates {
            self.apply_detected_models(presets, window, cx);
        }

        // スクロール更新
        self.scroll_manager.update();

        let toolbar = self.render_toolbar(cx);

        // Messages area with chat bubbles (scrollable container)
        // try_lockを使ってデッドロックを防ぐ
        let messages = self.conversation_service.current_messages();

        // スクロール更新（新しいメッセージがある場合は最下部へ）
        self.scroll_manager.update();

        // スクロールコンテナ - overflow_hiddenとtrackをネストして実現
        let msgs_container = div().flex_1().overflow_hidden().child(
            div()
                .id("chat_scroll_area")
                .size_full()
                .overflow_y_scroll()
                .track_scroll(self.scroll_manager.handle())
                .child(
                    div()
                        .v_flex()
                        .p_4()
                        .gap_3()
                        .children(messages.iter().map(|msg| {
                            let msg_type = match msg.role {
                                MessageRole::User => MessageType::User,
                                MessageRole::Assistant => MessageType::Assistant,
                                MessageRole::System => MessageType::System,
                                MessageRole::Error => MessageType::Error,
                            };

                            // メッセージタイプに応じて左右に配置
                            match msg_type {
                                MessageType::User => {
                                    // ユーザーメッセージは右寄せ
                                    div()
                                        .flex()
                                        .justify_end()
                                        .child(div().max_w(px(600.0)).child(
                                            ChatBubble::new(msg.content.clone(), msg_type).render(),
                                        ))
                                }
                                _ => {
                                    // それ以外は左寄せ
                                    div().flex().justify_start().child(
                                        div().max_w(px(600.0)).child(
                                            ChatBubble::new(msg.content.clone(), msg_type).render(),
                                        ),
                                    )
                                }
                            }
                        })),
                ),
        );

        let model_controls = self.render_model_selector_row();

        // Input area with multi-line input support
        let input_area = div().p_4().border_t_1().border_color(rgb(0x333333)).child(
            div()
                .v_flex()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x888888))
                        .child("Tip: Enter to send, Shift+Enter for new line"),
                )
                .child(Input::new(&self.input_state).w_full()),
        );

        // 現在の会話IDを取得
        let current_id = self
            .conversation_service
            .current_conversation_id()
            .unwrap_or_default();

        // サイドバーを描画
        let sidebar = sidebar::render_sidebar(
            &self.conversations_list,
            &current_id,
            |this: &mut Self, _event, window, cx| {
                this.create_new_conversation(_event, window, cx);
            },
            |this: &mut Self, conv_id: &str, window, cx| {
                this.switch_conversation(conv_id, window, cx);
            },
            |this: &mut Self, conv_id: &str, cx| {
                this.delete_conversation(conv_id, cx);
            },
            cx,
        );

        // メインコンテンツエリア
        let main_content = div()
            .flex_1()
            .h_full()
            .v_flex()
            .child(toolbar)
            .child(msgs_container)
            .child(model_controls)
            .child(input_area);

        let editor_console = self.render_editor_console();

        let workspace = div()
            .flex_1()
            .h_full()
            .h_flex()
            .child(editor_console)
            .child(main_content);

        div()
            .h_flex()
            .w_full()
            .h_full()
            .child(sidebar)
            .child(workspace)
    }
}

fn describe_agent_mode(mode: PromptAgentMode) -> &'static str {
    match mode {
        PromptAgentMode::LangChain => "LangChain 経由",
        PromptAgentMode::DirectProvider => "Direct Provider",
    }
}

pub fn run_gui(repo_root: &Path) -> std::io::Result<()> {
    // Discover plugins
    let list = crate::plugins::discover_plugins(repo_root)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let mut registry = PromptBuilderRegistry::from_plugins(&list);
    prompt_builders::register_builtin_prompt_builders(&mut registry);
    let prompt_registry = Arc::new(registry);

    // take ownership of repo_root for async usage
    let repo_root_buf = repo_root.to_path_buf();

    Application::new().run(move |cx: &mut App| {
        // Initialize gpui-component once.
        gpui_component::init(cx);

        // clone owned handles for the async task
        let list_clone = list.clone();
        let repo_clone = repo_root_buf.clone();
        let registry_clone = prompt_registry.clone();

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), move |window, cx| {
                // Main window: chat view with toolbar that can open plugin manager
                let view = cx.new(|cx| {
                    ChatView::new(
                        window,
                        cx,
                        repo_clone.clone(),
                        list_clone.clone(),
                        registry_clone.clone(),
                    )
                });
                cx.new(|cx| Root::new(view, window, cx))
            })
            .unwrap();
        })
        .detach();

        cx.activate(true);
    });

    Ok(())
}
