use gpui::*;

use crate::chat_message_list::{chat_message_list, ChatMessageRow};

fn with_overflow_y_scroll(mut node: Stateful<Div>) -> Stateful<Div> {
    node.style().overflow.y = Some(Overflow::Scroll);
    node
}

/// スクロール付きのチャットメッセージパネル
pub fn chat_messages_panel(
    scroll_id: &str,
    scroll_handle: &ScrollHandle,
    rows: &[ChatMessageRow],
) -> Div {
    let scroll_id = SharedString::from(scroll_id.to_owned());
    div().flex_1().overflow_hidden().child(
        with_overflow_y_scroll(div().id(scroll_id).size_full())
            .track_scroll(scroll_handle)
            .child(chat_message_list(rows)),
    )
}
