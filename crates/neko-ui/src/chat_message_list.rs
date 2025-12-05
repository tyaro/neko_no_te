use gpui::*;
use gpui_component::StyledExt;

use crate::chat_bubble::{ChatBubble, MessageType};

/// 表示用のチャットメッセージ行
#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessageRow {
    pub content: String,
    pub message_type: MessageType,
    pub align_end: bool,
    pub is_thinking: bool,
    pub source_label: Option<String>,
}

/// チャットメッセージリスト
pub fn chat_message_list(rows: &[ChatMessageRow]) -> Div {
    div()
        .v_flex()
        .h_full()
        .p_4()
        .gap_3()
        .children(rows.iter().map(|row| {
            let bubble = if row.is_thinking {
                ChatBubble::thinking_placeholder().into_any_element()
            } else {
                ChatBubble::new(row.content.clone(), row.message_type.clone())
                    .render()
                    .into_any_element()
            };

            // make bubble container fill the available width so bubble matches chat window width
            let mut bubble_container = div().max_w_4_5().v_flex().gap_1();
            if let Some(label) = &row.source_label {
                bubble_container = bubble_container.child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xa5b4fc))
                        .child(label.clone()),
                );
            }
            let bubble_container = bubble_container.child(bubble);

            if row.align_end {
                div().flex().justify_end().child(bubble_container)
            } else {
                div().flex().justify_start().child(bubble_container)
            }
        }))
}
