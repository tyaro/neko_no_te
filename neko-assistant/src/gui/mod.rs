pub mod console;
pub mod gpui;

pub use console::run_gui as run_gui_console;
pub use gpui::run_gui as run_gui_gpui;

#[allow(dead_code)]
pub fn run_gui(repo_root: &std::path::Path) -> std::io::Result<()> {
    // Choose implementation by feature
    #[cfg(feature = "gui")]
    {
        return run_gui_gpui(repo_root);
    }

    #[cfg(not(feature = "gui"))]
    {
        return run_gui_console(repo_root);
    }
}
