use gpui::*;
use gpui_component::button::Button;
use gpui_component::StyledExt;

/// チャットビュー上部のツールバー
pub fn chat_toolbar(
    plugins_button: Button,
    settings_button: Button,
    manage_mcp_button: Button,
    builder_status: impl Into<SharedString>,
) -> Div {
    div()
        .h_flex()
        .gap_2()
        .p_2()
        .child(plugins_button)
        .child(settings_button)
        .child(manage_mcp_button)
        .child(
            div()
                .flex_1()
                .justify_end()
                .text_sm()
                .text_color(rgb(0x888888))
                .child(builder_status.into()),
        )
}
