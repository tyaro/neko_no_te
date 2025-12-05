use gpui::*;
use gpui_component::select::SelectState;
use gpui_component::StyledExt;

use crate::model_selector::{model_selector, ModelPreset};

/// モデルセレクタのラベルと入力行をまとめた行コンポーネント
pub fn model_selector_row(
    select_state: &Entity<SelectState<Vec<ModelPreset>>>,
    has_prompt_builder: bool,
    has_adapter: bool,
) -> Div {
    div()
        .border_t_1()
        .border_color(rgb(0x333333))
        .bg(rgb(0x101010))
        .p_2()
        .v_flex()
        .gap_1()
        .child(div().text_sm().text_color(rgb(0xaaaaaa)).child("Model"))
        .child(
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .child(model_selector(select_state))
                .child(
                    div().h_flex().gap_2().items_center()
                        // Existing prompt-builder badge
                        .child(div()
                            .text_xs()
                            .rounded(px(6.0))
                            .p_1()
                            .bg(if has_prompt_builder { rgb(0x153f00) } else { rgb(0x2b2b2b) })
                            .text_color(rgb(0xffffff))
                            .child(if has_prompt_builder { "専用プロンプトあり" } else { "専用プロンプト無し" })
                        )
                        // Adapter/plugin badge (new)
                        .child(div()
                            .text_xs()
                            .rounded(px(6.0))
                            .p_1()
                            .bg(if has_adapter { rgb(0x153f00) } else { rgb(0x2b2b2b) })
                            .text_color(rgb(0xffffff))
                            .child(if has_adapter { "アダプタあり" } else { "アダプタ無し" })
                        ),
                ),
        )
}
