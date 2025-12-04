pub mod chat;
pub mod console;
pub mod mcp_manager;
pub mod plugins;
pub mod settings;

use gpui::{SharedString, TitlebarOptions, WindowOptions};

pub use chat::run_gui as run_gui_gpui;
pub use console::run_gui as run_gui_console;
pub use plugins::PluginListView;

/// Helper to create a window options struct with a simple titlebar configuration.
pub fn window_options_with_title(title: impl Into<SharedString>) -> WindowOptions {
    WindowOptions {
        titlebar: Some(TitlebarOptions {
            title: Some(title.into()),
            ..Default::default()
        }),
        ..WindowOptions::default()
    }
}

#[allow(dead_code)]
pub fn run_gui(repo_root: &std::path::Path) -> std::io::Result<()> {
    // Prefer the GPUI implementation at runtime; if it fails, fall back to console.
    match run_gui_gpui(repo_root) {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("GUI failed to start ({}). Falling back to console.", e);
            run_gui_console(repo_root)
        }
    }
}
