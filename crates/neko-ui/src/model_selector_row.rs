use gpui::*;
use gpui_component::input::InputState;
use gpui_component::select::SelectState;
use gpui_component::StyledExt;

use crate::model_selector::{model_selector, ModelPreset};

/// モデルセレクタのラベルと入力行をまとめた行コンポーネント
pub fn model_selector_row(
    select_state: &Entity<SelectState<Vec<ModelPreset>>>,
    selector_input: &Entity<InputState>,
) -> Div {
    div()
        .border_t_1()
        .border_color(rgb(0x333333))
        .bg(rgb(0x101010))
        .p_2()
        .v_flex()
        .gap_1()
        .child(div().text_sm().text_color(rgb(0xaaaaaa)).child("Model"))
        .child(model_selector(select_state, selector_input))
}
