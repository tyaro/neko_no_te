use super::menu_context::MenuContext;
use super::ChatView;
use crate::gui::{mcp_manager, window_options_with_title, PluginListView};
use chat_core::ChatCommand;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::menu::{DropdownMenu, PopupMenu, PopupMenuItem};
use gpui_component::Root;
use std::sync::Arc;

pub fn app_menu_button(
    context: &MenuContext,
    view_entity: gpui::Entity<ChatView>,
) -> impl IntoElement {
    let controller = context.controller();
    let repo_root = context.repo_root();
    let plugins = context.plugins();
    let toggle_label = context.mcp_toggle_label();

    Button::new("main_menu_dropdown")
        .label("Menu")
        .dropdown_menu(
            move |mut menu: PopupMenu,
                  window: &mut gpui::Window,
                  _popup_cx: &mut gpui::Context<PopupMenu>| {
                let controller_for_manager = controller.clone();
                let repo_for_plugins = repo_root.clone();
                let plugins_for_plugins = plugins.clone();
                let view_for_scratchpad = view_entity.clone();
                let view_for_console = view_entity.clone();
                let view_for_toggle = view_entity.clone();
                let toggle_label = toggle_label.clone();

                menu = menu.item(
                    PopupMenuItem::new("Settings").on_click(|_, _window, app_cx| {
                        crate::gui::settings::open_settings_window(app_cx);
                    }),
                );

                menu = menu.item(PopupMenuItem::new("MCP Manager").on_click(
                    move |_, _window, app_cx| {
                        let controller_for_refresh = controller_for_manager.clone();
                        let refresh_callback: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
                            if let Err(err) = controller_for_refresh
                                .handle_command(ChatCommand::RefreshMcpMetadata)
                            {
                                eprintln!(
                                    "Failed to refresh MCP metadata from manager: {}",
                                    err.message()
                                );
                            }
                        });
                        mcp_manager::open_mcp_manager_window(app_cx, Some(refresh_callback));
                    },
                ));

                menu = menu.item(PopupMenuItem::new("Plugins").on_click(
                    move |_, _window, app_cx| {
                        let repo_clone = repo_for_plugins.clone();
                        let plugins_clone = plugins_for_plugins.clone();
                        let _ = app_cx.open_window(
                            window_options_with_title("Plugins"),
                            move |window, cx| {
                                let view = cx.new(|_| {
                                    PluginListView::new(
                                        repo_clone.as_ref(),
                                        (*plugins_clone).clone(),
                                    )
                                });
                                cx.new(|cx| Root::new(view, window, cx))
                            },
                        );
                    },
                ));

                menu = menu.separator();

                menu = menu.item(
                    PopupMenuItem::new("Scratchpad").on_click(window.listener_for(
                        &view_for_scratchpad,
                        |this: &mut ChatView,
                         _event: &ClickEvent,
                         window,
                         cx: &mut gpui::Context<ChatView>| {
                            this.open_scratchpad_sheet(window, cx);
                        },
                    )),
                );

                menu = menu.item(PopupMenuItem::new("Console").on_click(window.listener_for(
                    &view_for_console,
                    |this: &mut ChatView,
                     _event: &ClickEvent,
                     window,
                     cx: &mut gpui::Context<ChatView>| {
                        this.open_console_sheet(window, cx);
                    },
                )));

                menu = menu.item(
                    PopupMenuItem::new(toggle_label).on_click(window.listener_for(
                        &view_for_toggle,
                        |this: &mut ChatView,
                         _event: &ClickEvent,
                         _window,
                         cx: &mut gpui::Context<ChatView>| {
                            this.state.toggle_mcp_status();
                            cx.notify();
                        },
                    )),
                );

                menu
            },
        )
}

pub fn manage_mcp_button(context: &MenuContext, id: &str, label: &str) -> Button {
    let controller = context.controller();
    Button::new(SharedString::from(id.to_string()))
        .label(SharedString::from(label.to_string()))
        .on_click(move |_, _window, app_cx| {
            let controller_for_refresh = controller.clone();
            let refresh_callback: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
                if let Err(err) =
                    controller_for_refresh.handle_command(ChatCommand::RefreshMcpMetadata)
                {
                    eprintln!(
                        "Failed to refresh MCP metadata from manager: {}",
                        err.message()
                    );
                }
            });
            mcp_manager::open_mcp_manager_window(app_cx, Some(refresh_callback));
        })
}

pub fn plugin_button(context: &MenuContext) -> Button {
    let repo_clone = context.repo_root();
    let plugins_clone = context.plugins();
    Button::new(SharedString::from("menu_plugins"))
        .label(SharedString::from("Plugins"))
        .on_click(move |_, _window, app_cx| {
            let repo_clone = repo_clone.clone();
            let plugins_clone = plugins_clone.clone();
            let _ = app_cx.open_window(window_options_with_title("Plugins"), move |window, cx| {
                let view =
                    cx.new(|_| PluginListView::new(repo_clone.as_ref(), (*plugins_clone).clone()));
                cx.new(|cx| Root::new(view, window, cx))
            });
        })
}
