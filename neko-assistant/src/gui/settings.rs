use crate::gui::window_options_with_title;
use gpui::*;
use gpui_component::button::*;
use gpui_component::input::{Input, InputState};
use gpui_component::{Root, StyledExt};
use std::cell::RefCell;
use std::rc::Rc;

/// 設定画面のビュー
pub struct SettingsView {
    ollama_url_input: gpui::Entity<InputState>,
    model_input: gpui::Entity<InputState>,
    max_history_input: gpui::Entity<InputState>,
    use_langchain: Rc<RefCell<bool>>,
    status_message: Rc<RefCell<Option<String>>>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl SettingsView {
    pub fn new(window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> Self {
        let config = app_config::AppConfig::load_or_default();

        // 入力フィールドの初期化
        let ollama_url_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_value(&config.ollama_base_url, window, cx);
            state
        });

        let model_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_value(&config.default_model, window, cx);
            state
        });

        let max_history_input = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            let history_value = config.max_history_messages.to_string();
            state.set_value(&history_value, window, cx);
            state
        });

        let use_langchain = Rc::new(RefCell::new(config.use_langchain));
        let status_message = Rc::new(RefCell::new(None));

        Self {
            ollama_url_input,
            model_input,
            max_history_input,
            use_langchain,
            status_message,
            _subscriptions: Vec::new(),
        }
    }
}

impl gpui::Render for SettingsView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        gpui_component::init(cx);

        let mut content = div().v_flex().gap_4().p_4().size_full();

        // タイトル
        content = content.child(div().child("Settings").text_size(px(24.0)));

        // Ollama URL
        content = content.child(
            div()
                .v_flex()
                .gap_2()
                .child(div().child("Ollama Base URL:"))
                .child(Input::new(&self.ollama_url_input)),
        );

        // Model Name
        content = content.child(
            div()
                .v_flex()
                .gap_2()
                .child(div().child("Default Model:"))
                .child(Input::new(&self.model_input)),
        );

        // Max History
        content = content.child(
            div()
                .v_flex()
                .gap_2()
                .child(div().child("Max History Messages:"))
                .child(Input::new(&self.max_history_input)),
        );

        // LangChain 使用設定（ボタンで切り替え）
        let use_langchain_ref = self.use_langchain.clone();
        let is_checked = *use_langchain_ref.borrow();
        let checkbox_text = if is_checked {
            "[✓] Use LangChain (experimental)"
        } else {
            "[ ] Use LangChain (experimental)"
        };

        content = content.child(
            Button::new("toggle_langchain")
                .label(checkbox_text)
                .on_click(cx.listener({
                    let use_langchain = use_langchain_ref.clone();
                    move |_this: &mut Self, _event, _window, cx| {
                        let mut val = use_langchain.borrow_mut();
                        *val = !*val;
                        cx.notify();
                    }
                })),
        );

        // 保存ボタン
        let status_msg = self.status_message.clone();
        let ollama_input = self.ollama_url_input.clone();
        let model_input = self.model_input.clone();
        let max_input = self.max_history_input.clone();
        let use_langchain = self.use_langchain.clone();

        content = content.child(
            div().h_flex().gap_2().child(
                Button::new("save_settings")
                    .label("Save Settings")
                    .on_click(cx.listener(move |_this: &mut Self, _event, _window, cx| {
                        // 入力値を取得
                        let ollama_url = ollama_input.read(cx).value().to_string();
                        let model = model_input.read(cx).value().to_string();
                        let max_history_str = max_input.read(cx).value().to_string();

                        // バリデーション
                        let max_history = match max_history_str.parse::<usize>() {
                            Ok(n) if n > 0 => n,
                            _ => {
                                *status_msg.borrow_mut() = Some(
                                    "Error: Max history must be a positive number".to_string(),
                                );
                                return;
                            }
                        };

                        // 設定を作成して保存
                        let mut config = app_config::AppConfig::load_or_default();
                        config.ollama_base_url = ollama_url;
                        config.default_model = model;
                        config.max_history_messages = max_history;
                        config.use_langchain = *use_langchain.borrow();

                        match config.save() {
                            Ok(_) => {
                                *status_msg.borrow_mut() =
                                    Some("Settings saved successfully!".to_string());
                            }
                            Err(e) => {
                                *status_msg.borrow_mut() = Some(format!("Error saving: {}", e));
                            }
                        }

                        cx.notify();
                    })),
            ),
        );

        // ステータスメッセージ
        if let Some(msg) = self.status_message.borrow().as_ref() {
            content = content.child(div().child(msg.clone()));
        }

        content
    }
}

/// 設定ウィンドウを開く
pub fn open_settings_window(cx: &mut App) {
    let _ = cx.open_window(window_options_with_title("Settings"), move |window, cx| {
        let view = cx.new(|cx| SettingsView::new(window, cx));
        cx.new(|cx| Root::new(view, window, cx))
    });
}
