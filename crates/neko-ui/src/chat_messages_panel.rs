use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::scroll::ScrollableElement;

use crate::chat_message_list::{chat_message_list, ChatMessageRow};

/// スクロール付きのチャットメッセージパネル（ScrollHandle不使用版）
pub fn chat_messages_panel(
    rows: &[ChatMessageRow],
    scroll_handle: Option<&gpui::ScrollHandle>,
) -> Div {
    div()
        .flex_1()
        .h_full()
        // .max_h(px(400.0))
        .overflow_hidden()
        .child(
            if let Some(handle) = scroll_handle {
                div()
                    .id("chat-messages-panel")
                    .track_scroll(handle)
                    .map(|this| this.overflow_y_scrollbar().child(chat_message_list(rows)))
            } else {
                div()
                    .id("chat-messages-panel")
                    .overflow_y_scrollbar()
                    .child(chat_message_list(rows))
            },
        )
}
