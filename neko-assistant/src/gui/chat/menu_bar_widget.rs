use super::menu::main_menu;
use super::menu_actions::{app_menu_button, manage_mcp_button, plugin_button};
use super::menu_context::MenuContext;
use super::ChatView;
use gpui::{IntoElement, ParentElement};

pub(super) fn menu_bar_widget(
    context: &MenuContext,
    view_entity: gpui::Entity<ChatView>,
) -> impl IntoElement {
    main_menu()
        .child(app_menu_button(context, view_entity.clone()))
        .child(manage_mcp_button(context, "menu_mcp", "MCP Manager"))
        .child(plugin_button(context))
}
