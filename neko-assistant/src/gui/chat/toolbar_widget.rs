use super::toolbar_view_model::ToolbarViewModel;
use super::ChatView;
use gpui::*;
use gpui_component::StyledExt;

pub(super) fn toolbar_widget(
    _view_entity: gpui::Entity<ChatView>,
    _view_model: ToolbarViewModel,
    _window: &mut gpui::Window,
) -> impl IntoElement {
    // toolbar no longer contains scratchpad/console/session buttons — those are in the top menu
    div().p_3().rounded_md().bg(rgb(0x101010)).child(div().h_flex().items_center())
}
