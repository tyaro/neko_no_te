use chat_core::discover_plugins;
use std::io;
use std::path::Path;

pub fn run_gui(repo_root: &Path) -> io::Result<()> {
    // Lightweight fallback: print plugin metadata to console as a preview.
    println!("GUI feature not enabled. Showing plugin metadata preview:\n");

    // Reuse the existing discovery logic to show metadata
    let list = discover_plugins(repo_root)?;
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
