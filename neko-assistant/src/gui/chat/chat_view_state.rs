use std::path::Path;

use super::model_selector::ModelSelector;
use chat_core::ChatState;
use gpui::{AppContext, Context, Entity, Subscription, Window};
use gpui_component::input::InputState;
use ui_utils::ScrollManager;

use super::{scratchpad::ScratchpadManager, ChatView};

/// ChatView 内の UI エンティティと関連状態を集約する構造体。
pub struct ChatViewState {
    model_selector: ModelSelector,
    input_state: Entity<InputState>,
    scratchpad: ScratchpadManager,
    scroll_manager: ScrollManager,
    show_mcp_status: bool,
    _subscriptions: Vec<Subscription>,
}

impl ChatViewState {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<ChatView>,
        chat_state: &ChatState,
        repo_root: &Path,
    ) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Type your message... (Enter to send)")
                .auto_grow(3, 10)
        });

        let model_selector = ModelSelector::new(
            window,
            cx,
            chat_state,
            ModelSelector::model_presets_from_state(chat_state),
        );

        let editor_input = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("markdown")
                .auto_grow(6, 16)
                .placeholder("Notes / prompt scratchpad...")
        });
        let scratchpad = ScratchpadManager::new(repo_root, editor_input.clone());

        Self {
            model_selector,
            input_state,
            scratchpad,
            scroll_manager: ScrollManager::new(),
            show_mcp_status: false,
            _subscriptions: Vec::new(),
        }
    }

    pub fn model_selector(&self) -> &ModelSelector {
        &self.model_selector
    }

    pub fn input_state(&self) -> &Entity<InputState> {
        &self.input_state
    }

    pub fn scratchpad(&self) -> &ScratchpadManager {
        &self.scratchpad
    }

    pub fn scroll_manager(&self) -> &ScrollManager {
        &self.scroll_manager
    }

    pub fn scroll_manager_mut(&mut self) -> &mut ScrollManager {
        &mut self.scroll_manager
    }

    pub fn mark_scroll_to_bottom(&mut self) {
        self.scroll_manager.mark_scroll_to_bottom();
    }

    pub fn show_mcp_status(&self) -> bool {
        self.show_mcp_status
    }

    pub fn toggle_mcp_status(&mut self) {
        self.show_mcp_status = !self.show_mcp_status;
    }

    pub fn set_subscriptions(&mut self, subs: Vec<Subscription>) {
        self._subscriptions = subs;
    }
}
