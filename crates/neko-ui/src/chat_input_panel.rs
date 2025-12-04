use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::StyledExt;

/// 入力ヒント付きのチャット入力パネル
pub fn chat_input_panel(input_state: &Entity<InputState>, hint_text: &str) -> Div {
    div()
        .w_full()
        .p_4()
        .border_t_1()
        .border_color(rgb(0x333333))
        .child(
            div()
                .w_full()
                .v_flex()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x888888))
                        .child(hint_text.to_string()),
                )
                .child(Input::new(input_state).w_full()),
        )
}
