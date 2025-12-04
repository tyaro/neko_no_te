use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::StyledExt;

trait OverflowScrollExt: Sized {
    fn overflow_y_scroll(self) -> Self;
}

impl OverflowScrollExt for Div {
    fn overflow_y_scroll(mut self) -> Self {
        self.style().overflow.y = Some(Overflow::Scroll);
        self
    }
}

/// ログ表示用のエントリ
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsoleLogEntry {
    pub role_label: String,
    pub content: String,
}

/// スクラッチパッドと簡易コンソールを並べて表示するコンポーネント
pub fn scratchpad_console(editor_input: &Entity<InputState>, logs: &[ConsoleLogEntry]) -> Div {
    let console_body: AnyElement = if logs.is_empty() {
        div()
            .flex_1()
            .justify_center()
            .items_center()
            .text_sm()
            .text_color(rgb(0x777777))
            .child("No messages yet")
            .into_any_element()
    } else {
        let items = logs.iter().map(|entry| {
            div()
                .text_xs()
                .text_color(rgb(0xcccccc))
                .child(format!("[{}] {}", entry.role_label, entry.content))
        });

        let scroll_body = div().v_flex().gap_1().children(items);

        div()
            .flex_1()
            .overflow_hidden()
            .child(div().size_full().overflow_y_scroll().child(scroll_body))
            .into_any_element()
    };

    div()
        .flex_1()
        .v_flex()
        .child(
            div()
                .p_2()
                .border_b_1()
                .border_color(rgb(0x333333))
                .v_flex()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xaaaaaa))
                        .child("Scratchpad"),
                )
                .child(Input::new(editor_input).w_full().h(px(200.0)).text_sm()),
        )
        .child(
            div()
                .p_2()
                .flex_1()
                .v_flex()
                .gap_1()
                .child(div().text_sm().text_color(rgb(0xaaaaaa)).child("Console"))
                .child(console_body),
        )
}
