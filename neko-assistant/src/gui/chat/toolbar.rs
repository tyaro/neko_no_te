use super::{mcp_manager, ChatView};
use gpui::*;
use gpui_component::button::Button;
use gpui_component::Root;
use gpui_component::StyledExt;

impl ChatView {
    pub(super) fn render_toolbar(&mut self, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let repo_clone = self.repo_root.clone();
        let plugins_clone = self.plugins.clone();
        let builder_status = self.describe_builder_status();

        div()
            .h_flex()
            .gap_2()
            .p_2()
            .child(
                Button::new(gpui::SharedString::from("open_plugins"))
                    .label(gpui::SharedString::from("Plugins"))
                    .on_click(move |_, _win, app_cx| {
                        let repo_clone = repo_clone.clone();
                        let plugins_clone = plugins_clone.clone();
                        let _ = app_cx.open_window(WindowOptions::default(), move |window, cx| {
                            let view = cx.new(|_| {
                                crate::gui::PluginListView::new(&repo_clone, plugins_clone.clone())
                            });
                            cx.new(|cx| Root::new(view, window, cx))
                        });
                    }),
            )
            .child(
                Button::new(gpui::SharedString::from("open_settings"))
                    .label(gpui::SharedString::from("Settings"))
                    .on_click(move |_, _win, app_cx| {
                        crate::gui::settings::open_settings_window(app_cx);
                    }),
            )
            .child(
                Button::new(gpui::SharedString::from("manage_mcp_servers"))
                    .label(gpui::SharedString::from("Manage MCP"))
                    .on_click(|_, _win, app_cx| {
                        mcp_manager::open_mcp_manager_window(app_cx);
                    }),
            )
            .child(
                div()
                    .flex_1()
                    .justify_end()
                    .text_sm()
                    .text_color(rgb(0x888888))
                    .child(builder_status),
            )
    }

    fn describe_builder_status(&self) -> String {
        if self.prompt_registry.is_empty() {
            return "Prompt Builder: 未検出".to_string();
        }

        if let Some(source) = self.prompt_registry.resolve(&self.active_model) {
            let meta = source.metadata();
            let mode = super::describe_agent_mode(source.preferred_agent());
            if let Some(manifest) = source.manifest() {
                let plugin_name = manifest.name.unwrap_or_else(|| meta.name.clone());
                let location = source
                    .plugin_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "(plugin)".into());
                return format!(
                    "Prompt Builder: {} v{} ({}, {}) @ {}",
                    plugin_name,
                    meta.version,
                    mode,
                    source.origin_label(),
                    location
                );
            }
            return format!(
                "Prompt Builder: {} v{} ({}, {})",
                meta.name,
                meta.version,
                mode,
                source.origin_label()
            );
        }

        format!("Prompt Builder: {} 用プラグインなし", self.active_model)
    }
}
