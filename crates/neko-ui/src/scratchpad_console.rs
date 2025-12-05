use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::scroll::ScrollableElement;
use gpui_component::StyledExt;

/// ログ表示用のエントリ
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsoleLogEntry {
    pub role_label: String,
    pub content: String,
}

/// スクラッチパッドと簡易コンソールを並べて表示するコンポーネント
pub fn scratchpad_console(
    editor_input: &Entity<InputState>,
    logs: &[ConsoleLogEntry],
    show_scratchpad: bool,
    show_console: bool,
) -> Div {
    let console_items: Vec<_> = if logs.is_empty() {
        vec![div()
            .text_xs()
            .text_color(rgb(0x777777))
            .child("No messages yet")]
    } else {
        logs.iter()
            .map(|entry| {
                div()
                    .text_xs()
                    .text_color(rgb(0xcccccc))
                    .child(format!("[{}] {}", entry.role_label, entry.content))
            })
            .collect()
    };

    let mut root = div().h_full().v_flex();

    if show_scratchpad {
            root = root.child(div().p_1()
                // make scratchpad larger than default by allowing it to grow
                .flex_1()
                .border_b_1()
                .border_color(rgb(0x333333))
                .v_flex()
                .gap_2()
                .child(
                    div()
                        .h_flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xaaaaaa))
                                .child("Scratchpad"),
                        ),
                )
                .child(Input::new(editor_input).w_full().h_full().text_sm()),
        );
    }
    if show_console {
        root = root.child(
            div()
                // keep console relatively small and fixed height
                .flex_shrink_0()
                .h(px(160.0))
            .v_flex()
            .p_1()
                .gap_1()
                .child(
                    div()
                        .flex_shrink_0()
                        .text_sm()
                        .text_color(rgb(0xaaaaaa))
                        .child("Console"),
                )
                .child(
                    div()
                        .overflow_hidden()
                        .child(
                            div()
                                .size_full()
                                .v_flex()
                                .gap_1()
                                .overflow_y_scrollbar()
                                .children(console_items),
                        ),
                ),
        );
    }

    root
}
// inline actions were removed: menu bar now toggles visibility and opens sheets
