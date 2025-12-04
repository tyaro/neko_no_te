use gpui::*;
use gpui_component::button::Button;
use gpui_component::StyledExt;

pub fn chat_window(
    chat_panel: impl IntoElement,
    model_controls: impl IntoElement,
    input_area: impl IntoElement,
    session_button: Button,
) -> Div {
    div()
        .v_flex()
        .gap_2()
        .p_3()
        .bg(rgb(0x0d0d0d))
        .child(
            div()
                .h_flex()
                .justify_between()
                .items_center()
                .child(div().text_lg().text_color(rgb(0xffffff)).child("Chat"))
                .child(session_button),
        )
        .child(chat_panel.into_element())
        .child(model_controls)
        .child(input_area)
}
