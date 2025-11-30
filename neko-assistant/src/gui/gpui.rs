use std::io;
use std::path::Path;

pub fn run_gui(repo_root: &Path) -> io::Result<()> {
    // Placeholder for a GPUI-powered GUI. Integrate the chosen GPUI crate here.
    // Example (pseudocode):
    // - initialize gpui app
    // - call discover_plugins(repo_root)
    // - render list with metadata and enable/disable toggles that call enable_plugin/disable_plugin

    println!("GUI feature enabled, but no GPUI implementation is provided in this template.");
    println!("Please add the chosen GPUI crate as an optional dependency and implement the UI in this function.\n");

    // Fallback preview: show same metadata as non-feature build.
    let list = crate::plugins::discover_plugins(repo_root)?;
    if list.is_empty() {
        println!("No plugins found in {}/plugins", repo_root.display());
        return Ok(());
    }

    for p in list {
        let meta = p.metadata;
        let title = meta
            .as_ref()
            .and_then(|m| m.name.clone())
            .unwrap_or_else(|| p.dir_name.clone());
        println!("Plugin: {}", title);
        if let Some(m) = meta {
            if let Some(d) = m.description {
                println!("  Description: {}", d);
            }
            if let Some(v) = m.version {
                println!("  Version: {}", v);
            }
            if let Some(a) = m.author {
                println!("  Author: {}", a);
            }
        }
        println!("  Path: {}", p.path.display());
        println!("  Enabled: {}\n", if p.enabled { "yes" } else { "no" });
    }

    Ok(())
}
