use gpui::*;
use gpui_component::button::Button;
use gpui_component::input::{Input, InputState};
use gpui_component::StyledExt;
use ui_utils::TextStyleExt;

pub fn scratchpad_window(
    editor_input: &Entity<InputState>,
    load_button: Button,
    save_button: Button,
) -> Div {
    div()
        .v_flex()
        .gap_2()
        .p_3()
        .bg(rgb(0x161616))
        .child(
            div()
                .text_md()
                .text_color(rgb(0xffffff))
                .child("Scratchpad"),
        )
        .child(div().h_flex().gap_2().child(load_button).child(save_button))
        .child(Input::new(editor_input).w_full().h(px(220.0)).text_sm())
}
