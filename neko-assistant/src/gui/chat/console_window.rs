use gpui::*;
use gpui_component::scroll::ScrollableElement;
use gpui_component::StyledExt;
use neko_ui::ConsoleLogEntry;
use ui_utils::TextStyleExt;

#[allow(dead_code)]
pub fn console_window(logs: &[ConsoleLogEntry]) -> Div {
    let console_items: Vec<_> = if logs.is_empty() {
        vec![div()
            .text_xs()
            .text_color(rgb(0x777777))
            .child("No console activity yet")]
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

    div().v_flex().gap_1().p_1()
        .bg(rgb(0x161616))
        .child(div().text_md().text_color(rgb(0xffffff)).child("Console"))
        .child(
            div()
                .h(px(300.0))
                .v_flex()
                .gap_1()
                .overflow_y_scrollbar()
                .children(console_items),
        )
}
