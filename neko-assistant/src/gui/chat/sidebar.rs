use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::StyledExt;

/// サイドバーの描画
pub fn render_sidebar<V: Render>(
    conversations_list: &[chat_history::ConversationMetadata],
    current_id: &str,
    on_new_chat: impl Fn(&mut V, &ClickEvent, &mut Window, &mut Context<V>) + 'static,
    on_conversation_click: impl Fn(&mut V, &str, &mut Window, &mut Context<V>) + 'static + Clone,
    on_delete_click: impl Fn(&mut V, &str, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> Div {
    div()
        .w(px(200.0))
        .h_full()
        .border_r_1()
        .border_color(rgb(0x333333))
        .v_flex()
        .child(
            // 新規会話ボタン
            div().p_2().border_b_1().border_color(rgb(0x333333)).child(
                Button::new(SharedString::from("new_chat"))
                    .label(SharedString::from("+ New Chat"))
                    .w_full()
                    .on_click(cx.listener(on_new_chat)),
            ),
        )
        .child(
            // 会話リスト
            div()
                .flex_1()
                .overflow_hidden()
                .child(
                    div()
                        .v_flex()
                        .p_2()
                        .gap_2()
                        .children(conversations_list.iter().map({
                            let on_click = on_conversation_click.clone();
                            let on_delete = on_delete_click.clone();
                            move |meta| {
                                let is_current = meta.id == current_id;
                                let title = SharedString::from(meta.title.clone());
                                let conv_id = meta.id.clone();
                                let on_click = on_click.clone();
                                let on_delete = on_delete.clone();
                                let delete_id = conv_id.clone();

                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .p_2()
                                    .rounded_md()
                                    .when(is_current, |div| div.bg(rgb(0x2a5a8a)))
                                    .when(!is_current, |div| {
                                        div.hover(|style| style.bg(rgb(0x3a3a3a)))
                                    })
                                    .child(
                                        // 会話情報部分（クリックで切り替え）
                                        div()
                                            .flex_1()
                                            .min_w(px(0.0))
                                            .cursor_pointer()
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, window, cx| {
                                                    if !is_current {
                                                        on_click(this, &conv_id, window, cx);
                                                    }
                                                }),
                                            )
                                            .child(
                                                div()
                                                    .v_flex()
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .overflow_hidden()
                                                            .child(title.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(0x888888))
                                                            .child(format!(
                                                                "{} messages",
                                                                meta.message_count
                                                            )),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // 削除ボタン（現在の会話以外は表示）
                                        div()
                                            .flex_shrink_0()
                                            .when(!is_current, |d| {
                                                d.p_1()
                                                    .rounded_sm()
                                                    .text_sm()
                                                    .text_color(rgb(0x888888))
                                                    .hover(|style| {
                                                        style
                                                            .text_color(rgb(0xff4444))
                                                            .bg(rgb(0x444444))
                                                    })
                                                    .cursor_pointer()
                                                    .on_mouse_down(
                                                        MouseButton::Left,
                                                        cx.listener(move |this, _, _window, cx| {
                                                            on_delete(this, &delete_id, cx);
                                                        }),
                                                    )
                                                    .child("×")
                                            })
                                            .when(is_current, |d| d), // 現在の会話の場合は空のdiv
                                    )
                            }
                        })),
                ),
        )
}
