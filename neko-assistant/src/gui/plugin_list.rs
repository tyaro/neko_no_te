//! プラグインリストビュー

use chat_core::PluginEntry;
use gpui::*;
use gpui_component::button::*;
use gpui_component::StyledExt;
use std::path::PathBuf;

pub struct PluginListView {
    _repo_root: PathBuf,
    plugins: Vec<PluginEntry>,
    selected: Option<usize>,
}

impl PluginListView {
    pub fn new(repo_root: &std::path::Path, plugins: Vec<PluginEntry>) -> Self {
        let selected = if !plugins.is_empty() { Some(0) } else { None };
        Self {
            _repo_root: repo_root.to_path_buf(),
            plugins,
            selected,
        }
    }
}

impl Render for PluginListView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Init gpui-component helpers (safe to call each frame)
        gpui_component::init(cx);

        // Left: list of plugins as buttons. Right: details for selected plugin.
        let list_col = {
            let mut col = div().v_flex().gap_2().size_full();
            for (_i, entry) in self.plugins.iter().enumerate() {
                let title = entry
                    .metadata
                    .as_ref()
                    .and_then(|m| m.name.clone())
                    .unwrap_or_else(|| entry.dir_name.clone());

                // Convert to SharedString so ElementId can be built from it
                let title_ss = SharedString::from(title.clone());

                // Render as simple buttons (no click wiring yet).
                let btn = Button::new(title_ss.clone()).label(title_ss.clone());
                col = col.child(btn);
            }
            col
        };

        let detail_col = if let Some(idx) = self.selected {
            if let Some(entry) = self.plugins.get(idx) {
                let name = entry
                    .metadata
                    .as_ref()
                    .and_then(|m| m.name.clone())
                    .unwrap_or_else(|| entry.dir_name.clone());
                let desc = entry
                    .metadata
                    .as_ref()
                    .and_then(|m| m.description.clone())
                    .unwrap_or_default();
                let name_ss = SharedString::from(name);
                let desc_ss = SharedString::from(desc);
                div()
                    .v_flex()
                    .gap_2()
                    .child(div().child(name_ss))
                    .child(div().child(desc_ss))
                    .child(Button::new("enable").label("Enable"))
            } else {
                div().child("No plugin selected")
            }
        } else {
            div().child("No plugin selected")
        };

        // Root layout: horizontal split
        div()
            .h_flex()
            .gap_4()
            .size_full()
            .child(list_col.flex_grow())
            .child(detail_col.flex_grow())
    }
}
