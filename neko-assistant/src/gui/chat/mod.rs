mod conversation_actions;
mod sidebar;

use super::mcp_manager;
use crate::conversation_service::ConversationService;
use crate::message_handler::MessageHandler;
use crate::plugins::PluginEntry;
use chat_history::{Conversation, Message, MessageRole};
use gpui::*;
use gpui_component::button::*;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::Root;
use gpui_component::StyledExt;
use neko_ui::{ChatBubble, MessageType};
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use ui_utils::ScrollManager;

use conversation_actions::ConversationActions;

// Chat view with proper chat bubbles
pub struct ChatView {
    repo_root: PathBuf,
    plugins: Vec<PluginEntry>,
    input_state: gpui::Entity<InputState>,
    conversation_service: ConversationService,
    conversations_list: Vec<chat_history::ConversationMetadata>,
    conversation_actions: ConversationActions,
    ui_update_rx: Arc<Mutex<mpsc::UnboundedReceiver<()>>>,
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
    ) -> Self {
        // 設定を読み込み
        let config = app_config::AppConfig::load_or_default();
        let use_langchain = config.use_langchain;

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
            config.default_model.clone(),
            mcp_manager,
        ));

        // ConversationActionsを初期化
        let conversation_actions = ConversationActions::new(conversation_service.clone());

        // create an InputState entity with multi-line support
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Type your message... (Enter to send)")
                .auto_grow(3, 10) // 3〜10行の自動成長
        });

        // Subscribe to input events
        let handler_sub = message_handler.clone();
        let subs = vec![cx.subscribe_in(
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

        Self {
            repo_root,
            plugins,
            input_state,
            conversation_service,
            conversations_list: Vec::new(), // 初期化時は空、render時に読み込む
            conversation_actions,
            ui_update_rx,
            scroll_manager: ScrollManager::new(),
            _message_handler: message_handler,
            _subscriptions: subs,
        }
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
}

impl gpui::Render for ChatView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
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

        // スクロール更新
        self.scroll_manager.update();

        // Toolbar with Plugins and Settings buttons
        let repo_clone = self.repo_root.clone();
        let plugins_clone = self.plugins.clone();
        let toolbar = div()
            .h_flex()
            .gap_2()
            .p_2()
            .child(
                Button::new(gpui::SharedString::from("open_plugins"))
                    .label(gpui::SharedString::from("Plugins"))
                    .on_click(move |_, _win, app_cx| {
                        let repo_clone = repo_clone.clone();
                        let plugins_clone = plugins_clone.clone();
                        let _ = app_cx.open_window(WindowOptions::default(), move |window, cx| {
                            let view = cx.new(|_| {
                                crate::gui::PluginListView::new(&repo_clone, plugins_clone.clone())
                            });
                            cx.new(|cx| Root::new(view, window, cx))
                        });
                    }),
            )
            .child(
                Button::new(gpui::SharedString::from("open_settings"))
                    .label(gpui::SharedString::from("Settings"))
                    .on_click(move |_, _win, app_cx| {
                        crate::gui::settings::open_settings_window(app_cx);
                    }),
            )
            .child(
                Button::new(gpui::SharedString::from("manage_mcp_servers"))
                    .label(gpui::SharedString::from("Manage MCP"))
                    .on_click(|_, _win, app_cx| {
                        mcp_manager::open_mcp_manager_window(app_cx);
                    }),
            );

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
            .child(input_area);

        div()
            .h_flex()
            .w_full()
            .h_full()
            .child(sidebar)
            .child(main_content)
    }
}

pub fn run_gui(repo_root: &Path) -> std::io::Result<()> {
    // Discover plugins
    let list = crate::plugins::discover_plugins(repo_root)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // take ownership of repo_root for async usage
    let repo_root_buf = repo_root.to_path_buf();

    Application::new().run(move |cx: &mut App| {
        // Initialize gpui-component once.
        gpui_component::init(cx);

        // clone owned handles for the async task
        let list_clone = list.clone();
        let repo_clone = repo_root_buf.clone();

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), move |window, cx| {
                // Main window: chat view with toolbar that can open plugin manager
                let view =
                    cx.new(|cx| ChatView::new(window, cx, repo_clone.clone(), list_clone.clone()));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .unwrap();
        })
        .detach();

        cx.activate(true);
    });

    Ok(())
}
