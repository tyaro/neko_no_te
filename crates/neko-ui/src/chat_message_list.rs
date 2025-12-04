use gpui::*;
use gpui_component::StyledExt;

use crate::chat_bubble::{ChatBubble, MessageType};

/// 表示用のチャットメッセージ行
#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessageRow {
    pub content: String,
    pub message_type: MessageType,
    pub align_end: bool,
}

/// チャットメッセージリスト
pub fn chat_message_list(rows: &[ChatMessageRow]) -> Div {
    div()
        .v_flex()
        .p_4()
        .gap_3()
        .children(rows.iter().map(|row| {
            let bubble = ChatBubble::new(row.content.clone(), row.message_type.clone()).render();
            let bubble_container = div().max_w(px(600.0)).child(bubble);

            if row.align_end {
                div().flex().justify_end().child(bubble_container)
            } else {
                div().flex().justify_start().child(bubble_container)
            }
        }))
}
