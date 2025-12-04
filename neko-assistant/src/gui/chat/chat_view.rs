use super::chat_view_state::ChatViewState;
use super::chat_window::chat_window;
use super::console_window::console_window;
use super::controller_facade::ChatControllerFacade;
use super::event_loop::ChatEventLoop;
use super::initialization::{ChatViewBuilder, ChatViewParts};
use super::menu_actions::manage_mcp_button;
use super::menu_bar_widget::menu_bar_widget;
use super::menu_context::MenuContext;
use super::session_popup::open_session_popup;
use super::toolbar_view_model::ToolbarViewModel;
use super::toolbar_widget::toolbar_widget;
use super::ui_state::ChatUiSnapshot;
use crate::gui::window_options_with_title;
use chat_core::{
    discover_plugins, register_builtin_prompt_builders, ChatCommand, ChatState, PluginEntry,
    PromptBuilderRegistry,
};
use gpui::*;
use gpui_component::button::Button;
use gpui_component::{Root, StyledExt, WindowExt};
use neko_ui::{chat_input_panel, chat_messages_panel, mcp_status_panel, model_selector_row};
use prompt_spi::PromptAgentMode;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct ChatView {
    pub(super) repo_root: PathBuf,
    pub(super) plugins: Vec<PluginEntry>,
    pub(super) prompt_registry: Arc<PromptBuilderRegistry>,
    pub(super) controller: ChatControllerFacade,
    pub(super) event_loop: ChatEventLoop,
    pub(super) state: ChatViewState,
}

impl ChatView {
    pub fn new(
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
        repo_root: PathBuf,
        plugins: Vec<PluginEntry>,
        prompt_registry: Arc<PromptBuilderRegistry>,
    ) -> Self {
        let parts = ChatViewBuilder::new(repo_root, plugins, prompt_registry).build(window, cx);
        let view = Self::from_parts(parts);

        if let Err(err) = view.controller.handle_command(ChatCommand::RefreshModels) {
            eprintln!("Failed to refresh model list: {}", err.message());
        }

        view
    }

    fn from_parts(parts: ChatViewParts) -> Self {
        Self {
            repo_root: parts.repo_root,
            plugins: parts.plugins,
            prompt_registry: parts.prompt_registry,
            controller: parts.controller,
            event_loop: parts.event_loop,
            state: parts.state,
        }
    }

    pub(super) fn chat_state_snapshot(&self) -> ChatState {
        self.controller.state_snapshot()
    }

    pub(super) fn open_scratchpad_sheet(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let logs = ChatUiSnapshot::from_state(&self.chat_state_snapshot()).console_logs;
        let view = cx.entity();
        self.state.scratchpad().open_sheet(view, logs, window, cx);
    }

    pub(super) fn open_console_sheet(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        let logs = ChatUiSnapshot::from_state(&self.chat_state_snapshot()).console_logs;

        let _ = window.open_sheet(cx, move |sheet, _window, _app| {
            sheet
                .title(div().text_sm().text_color(rgb(0xffffff)).child("Console"))
                .size(px(380.0))
                .child(console_window(&logs))
        });
    }
}

impl Render for ChatView {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        gpui_component::init(cx);
        let event_loop = self.event_loop.clone();
        let batch = event_loop.poll();
        batch.apply(self, window, cx);

        let view_entity = cx.entity();
        let menu_context = MenuContext::from_chat_view(self);
        let state = self.chat_state_snapshot();
        let ui_snapshot = ChatUiSnapshot::from_state(&state);
        self.state.scroll_manager_mut().update();

        let msgs_container = chat_messages_panel(
            "chat_scroll_area",
            self.state.scroll_manager().handle(),
            &ui_snapshot.message_rows,
        );

        let toolbar_model = ToolbarViewModel::from_chat_view(self);
        let toolbar = toolbar_widget(view_entity.clone(), toolbar_model, window);
        let selector = self.state.model_selector();
        let model_controls = model_selector_row(selector.select_state(), selector.input_state());
        let input_area = chat_input_panel(
            self.state.input_state(),
            "Enter: send, Shift+Enter: newline",
        );

        let server_items = &ui_snapshot.server_items;
        let tool_items = &ui_snapshot.tool_items;

        let controller_for_refresh = menu_context.controller();
        let refresh_button =
            Button::new("mcp_refresh_button")
                .label("Refresh")
                .on_click(cx.listener(move |_this, _event, _window, _cx| {
                    if let Err(err) =
                        controller_for_refresh.handle_command(ChatCommand::RefreshMcpMetadata)
                    {
                        eprintln!("Failed to refresh MCP metadata: {}", err.message());
                    }
                }));

        let manage_button_inline = manage_mcp_button(&menu_context, "inline_mcp_manage", "Manage");

        let mcp_panel = if self.state.show_mcp_status() {
            Some(
                mcp_status_panel(
                    &server_items,
                    &tool_items,
                    refresh_button,
                    manage_button_inline,
                )
                .flex_shrink_0(),
            )
        } else {
            None
        };

        let menu_context_for_sessions = menu_context.clone();
        let session_button = Button::new("open_session_popup")
            .label("Sessions")
            .on_click(cx.listener(move |this, _event, _window, cx| {
                let snapshot = this.chat_state_snapshot();
                let sessions = ChatUiSnapshot::from_state(&snapshot).sidebar_items;
                open_session_popup(cx, &menu_context_for_sessions, sessions);
            }));

        let mut chat_body = div().flex_1().min_h(px(0.0)).v_flex().child(toolbar);
        if let Some(panel) = mcp_panel {
            chat_body = chat_body.child(panel);
        }
        let chat_body = chat_body.child(div().flex_1().min_h(px(0.0)).child(msgs_container));

        let chat_panel = chat_window(chat_body, model_controls, input_area, session_button)
            .flex_1()
            .h_full();

        let view_entity = cx.entity();
        let menu_bar = menu_bar_widget(&menu_context, view_entity.clone());

        let workspace = div()
            .flex_1()
            .h_flex()
            .justify_center()
            .px(px(24.0))
            .py(px(16.0))
            .child(div().flex_1().max_w(px(1100.0)).h_full().child(chat_panel));

        let mut root_layout = div().size_full().v_flex().child(menu_bar).child(workspace);

        if let Some(sheet_layer) = Root::render_sheet_layer(window, cx) {
            root_layout = root_layout.child(sheet_layer);
        }

        root_layout
    }
}

pub fn describe_agent_mode(mode: PromptAgentMode) -> &'static str {
    match mode {
        PromptAgentMode::LangChain => "LangChain 経由",
        PromptAgentMode::DirectProvider => "Direct Provider",
    }
}

pub fn run_gui(repo_root: &Path) -> std::io::Result<()> {
    let list = discover_plugins(repo_root)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let mut registry = PromptBuilderRegistry::from_plugins(&list);
    register_builtin_prompt_builders(&mut registry);
    let prompt_registry = Arc::new(registry);

    let repo_root_buf = repo_root.to_path_buf();

    Application::new().run(move |cx: &mut App| {
        gpui_component::init(cx);

        let list_clone = list.clone();
        let repo_clone = repo_root_buf.clone();
        let registry_clone = prompt_registry.clone();

        cx.spawn(async move |cx| {
            cx.open_window(
                window_options_with_title("Neko Assistant"),
                move |window, cx| {
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
                },
            )
            .unwrap();
        })
        .detach();

        cx.activate(true);
    });

    Ok(())
}
