use super::toolbar_view_model::ToolbarViewModel;
use super::ChatView;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::StyledExt;

pub(super) fn toolbar_widget(
    view_entity: gpui::Entity<ChatView>,
    view_model: ToolbarViewModel,
    window: &mut gpui::Window,
) -> impl IntoElement {
    let builder_status = SharedString::from(view_model.builder_status().to_owned());
    let scratchpad_listener = window.listener_for(
        &view_entity,
        |this: &mut ChatView, _event: &ClickEvent, window, cx| {
            this.open_scratchpad_sheet(window, cx);
        },
    );
    let console_listener = window.listener_for(
        &view_entity,
        |this: &mut ChatView, _event: &ClickEvent, window, cx| {
            this.open_console_sheet(window, cx);
        },
    );
    let toggle_listener = window.listener_for(
        &view_entity,
        |this: &mut ChatView, _event: &ClickEvent, _window, cx| {
            this.state.toggle_mcp_status();
            cx.notify();
        },
    );

    let scratchpad_button = Button::new("toolbar_scratchpad")
        .label("Scratchpad")
        .on_click(scratchpad_listener);
    let console_button = Button::new("toolbar_console")
        .label("Console")
        .on_click(console_listener);
    let mcp_toggle_button = Button::new("toolbar_mcp_toggle")
        .label(view_model.mcp_toggle_label())
        .on_click(toggle_listener);

    div().p_3().rounded_md().bg(rgb(0x101010)).child(
        div()
            .h_flex()
            .items_center()
            .justify_between()
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xaaaaaa))
                    .child(builder_status),
            )
            .child(
                div()
                    .h_flex()
                    .gap_2()
                    .child(scratchpad_button)
                    .child(console_button)
                    .child(mcp_toggle_button),
            ),
    )
}
