pub mod console;
pub mod chat;
pub mod settings;
pub mod plugins;

pub use console::run_gui as run_gui_console;
pub use chat::run_gui as run_gui_gpui;
pub use plugins::PluginListView;

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
