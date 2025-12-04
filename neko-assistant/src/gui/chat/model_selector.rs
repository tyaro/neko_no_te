use super::controller_facade::ChatControllerFacade;
use super::ChatView;
use app_config::AppConfig;
use chat_core::{ChatCommand, ChatState};
use gpui::{AppContext, Context, Entity, Window};
use gpui_component::input::InputState;
use gpui_component::select::SelectState;
use neko_ui::ModelPreset;

/// 管理者: モデルセレクターの入力/選択状態と同期処理をまとめる
pub struct ModelSelector {
    select_state: Entity<SelectState<Vec<ModelPreset>>>,
    input_state: Entity<InputState>,
}

impl ModelSelector {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<ChatView>,
        chat_state: &ChatState,
        model_presets: Vec<ModelPreset>,
    ) -> Self {
        let input_state = cx.new(|cx| {
            let mut state =
                InputState::new(window, cx).placeholder("モデルID (例: phi4-mini:3.8b)");
            state.set_value(&chat_state.active_model, window, cx);
            state
        });

        let selected_value = if model_presets
            .iter()
            .any(|preset| preset.id == chat_state.active_model)
        {
            Some(chat_state.active_model.clone())
        } else {
            None
        };

        let select_state = cx.new(|cx| {
            let mut state = SelectState::new(model_presets.clone(), None, window, cx);
            if let Some(value) = selected_value.clone() {
                state.set_selected_value(&value, window, cx);
            }
            state
        });

        Self {
            select_state,
            input_state,
        }
    }

    pub fn input_state(&self) -> &Entity<InputState> {
        &self.input_state
    }

    pub fn select_state(&self) -> &Entity<SelectState<Vec<ModelPreset>>> {
        &self.select_state
    }

    pub fn sync_items(&self, state: &ChatState, window: &mut Window, cx: &mut Context<ChatView>) {
        let presets = Self::model_presets_from_state(state);
        let active = state.active_model.clone();
        let has_match = presets.iter().any(|preset| preset.id == active);
        let _ = self.select_state.update(cx, |select, cx| {
            select.set_items(presets, window, cx);
            if has_match {
                select.set_selected_value(&active, window, cx);
            } else {
                select.set_selected_index(None, window, cx);
            }
        });
    }

    pub fn sync_selection(
        &self,
        state: &ChatState,
        window: &mut Window,
        cx: &mut Context<ChatView>,
    ) {
        let active = state.active_model.clone();
        let presets = Self::model_presets_from_state(state);
        let has_match = presets.iter().any(|preset| preset.id == active);
        let _ = self.select_state.update(cx, |select, cx| {
            if has_match {
                select.set_selected_value(&active, window, cx);
            } else {
                select.set_selected_index(None, window, cx);
            }
        });
    }

    pub fn update_input_value(&self, value: &str, window: &mut Window, cx: &mut Context<ChatView>) {
        let owned = value.to_string();
        let _ = self.input_state.update(cx, |state, cx| {
            state.set_value(&owned, window, cx);
        });
    }

    pub fn model_presets_from_state(state: &ChatState) -> Vec<ModelPreset> {
        state
            .available_models
            .iter()
            .map(|model| ModelPreset::new(model.id.clone(), model.label.clone()))
            .collect()
    }

    pub fn switch_model(
        &self,
        controller: &ChatControllerFacade,
        model_id: &str,
        window: &mut Window,
        cx: &mut Context<ChatView>,
    ) -> Result<(), String> {
        let normalized = model_id.trim();
        if normalized.is_empty() {
            return Ok(());
        }

        let current_model = controller.state_snapshot().active_model;
        if current_model == normalized {
            self.update_input_value(&current_model, window, cx);
            return Ok(());
        }

        controller
            .handle_command(ChatCommand::SwitchModel(normalized.to_string()))
            .map_err(|err| err.message().to_string())?;

        self.update_input_value(normalized, window, cx);
        Self::persist_model_selection(normalized)?;
        let state = controller.state_snapshot();
        self.sync_selection(&state, window, cx);
        cx.notify();
        Ok(())
    }

    fn persist_model_selection(model: &str) -> Result<(), String> {
        let mut config = AppConfig::load_or_default();
        config.default_model = model.to_string();
        config.save().map_err(|e| e.to_string())
    }
}
