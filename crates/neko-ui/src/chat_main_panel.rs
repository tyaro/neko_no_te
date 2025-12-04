use gpui::*;
use gpui_component::StyledExt;

/// チャットのメインペイン（ツールバー + メッセージ + モデル制御 + 入力）
pub fn chat_main_panel(
    toolbar: impl IntoElement,
    messages_panel: impl IntoElement,
    model_controls: impl IntoElement,
    input_panel: impl IntoElement,
) -> Div {
    div()
        .flex_1()
        .h_full()
        .v_flex()
        .child(toolbar)
        .child(messages_panel)
        .child(model_controls)
        .child(input_panel)
}
