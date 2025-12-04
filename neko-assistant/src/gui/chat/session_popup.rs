use super::menu_context::MenuContext;
use crate::gui::window_options_with_title;
use chat_core::{ChatCommand, ChatController};
use gpui::*;
use gpui_component::button::Button;
use gpui_component::{Root, StyledExt};
use neko_ui::ChatSidebarItem;
use std::sync::Arc;

pub struct SessionPopupView {
    controller: Arc<ChatController>,
    sessions: Vec<ChatSidebarItem>,
}

impl SessionPopupView {
    pub fn new(controller: Arc<ChatController>, sessions: Vec<ChatSidebarItem>) -> Self {
        Self {
            controller,
            sessions,
        }
    }
}

impl Render for SessionPopupView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        gpui_component::init(cx);

        let mut list = div().v_flex().gap_2();

        if self.sessions.is_empty() {
            list = list.child(
                div()
                    .text_sm()
                    .text_color(rgb(0xaaaaaa))
                    .child("No sessions yet"),
            );
        } else {
            for session in self.sessions.iter() {
                let controller = self.controller.clone();
                let session_id = session.id.clone();
                let button_id = session_id.clone();
                let title = session.title.clone();
                let message_count = session.message_count;
                let handler =
                    cx.listener(move |_this: &mut Self, _ev: &ClickEvent, window, _cx| {
                        if let Err(err) = controller
                            .handle_command(ChatCommand::SwitchConversation(session_id.clone()))
                        {
                            eprintln!("Failed to switch session: {}", err.message());
                        }
                        window.remove_window();
                    });

                list = list.child(
                    Button::new(SharedString::from(format!("session_{}", button_id)))
                        .label(format!("{} ({} msgs)", title, message_count))
                        .on_click(handler),
                );
            }
        }

        let controller = self.controller.clone();
        let create_handler = cx.listener(move |_this: &mut Self, _ev: &ClickEvent, window, _cx| {
            if let Err(err) = controller.handle_command(ChatCommand::CreateConversation) {
                eprintln!("Failed to create session: {}", err.message());
            }
            window.remove_window();
        });

        div()
            .p_3()
            .v_flex()
            .gap_3()
            .child(
                div()
                    .text_lg()
                    .text_color(rgb(0xffffff))
                    .child("Select Session"),
            )
            .child(list)
            .child(
                Button::new("create_session")
                    .label("New Session")
                    .on_click(create_handler),
            )
    }
}

pub fn open_session_popup(
    cx: &mut Context<super::ChatView>,
    context: &MenuContext,
    sessions: Vec<ChatSidebarItem>,
) {
    let controller = context.controller();
    let _ = cx.open_window(window_options_with_title("Sessions"), move |window, cx| {
        let view = cx.new(|_cx| SessionPopupView::new(controller.clone(), sessions.clone()));
        cx.new(|cx| Root::new(view, window, cx))
    });
}
