use gpui::*;
use gpui_component::StyledExt;

pub fn chat_window(
    chat_body: impl IntoElement,
    model_controls: impl IntoElement,
    input_area: impl IntoElement,
) -> Div {
    div()
        .h_full()
        .v_flex()
        .gap_1()
        .p_1()
        .bg(rgb(0x0d0d0d))
        .child(
            div()
                .flex_shrink_0()
                .h_flex()
                .justify_between()
                .items_center()
                .child(div().text_lg().text_color(rgb(0xffffff)).child("Chat"))
                // session button removed; sessions handled via top menu
        )
        .child(
            div()
                .flex_1()
                .min_h(px(0.0))
                .child(chat_body.into_element()),
        )
        .child(div().flex_shrink_0().child(model_controls))
        .child(div().flex_shrink_0().child(input_area))
}
