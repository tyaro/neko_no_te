use gpui::*;
use gpui_component::StyledExt;
use neko_ui::ConsoleLogEntry;
use ui_utils::TextStyleExt;

trait OverflowExt: Sized {
    fn overflow_y_scroll(self) -> Self;
}

impl OverflowExt for Div {
    fn overflow_y_scroll(mut self) -> Self {
        self.style().overflow.y = Some(Overflow::Scroll);
        self
    }
}

pub fn console_window(logs: &[ConsoleLogEntry]) -> Div {
    let body: AnyElement = if logs.is_empty() {
        div()
            .flex_1()
            .justify_center()
            .items_center()
            .text_sm()
            .text_color(rgb(0x777777))
            .child("No console activity yet")
            .into_any_element()
    } else {
        let entries = logs.iter().map(|entry| {
            div()
                .text_xs()
                .text_color(rgb(0xcccccc))
                .child(format!("[{}] {}", entry.role_label, entry.content))
        });

        div()
            .flex_1()
            .overflow_hidden()
            .child(
                div()
                    .size_full()
                    .overflow_y_scroll()
                    .child(div().v_flex().gap_1().children(entries)),
            )
            .into_any_element()
    };

    div()
        .v_flex()
        .gap_2()
        .p_3()
        .bg(rgb(0x161616))
        .child(div().text_md().text_color(rgb(0xffffff)).child("Console"))
        .child(body)
}
