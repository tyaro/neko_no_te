use crate::plugins::metadata::{PluginEntry, PluginMetadata};
use crate::plugins::enabled::load_enabled_list;
use std::fs;
use std::io;
use std::path::Path;

/// Discover plugin directories under the repository `plugins/` directory.
/// If a `plugin.toml` is present in each plugin dir it will be parsed and returned as metadata.
pub fn discover_plugins(repo_root: &Path) -> io::Result<Vec<PluginEntry>> {
    // Prefer a `plugins/` folder next to the running executable (development/runtime),
    // otherwise fall back to the repository `plugins/` directory passed as `repo_root`.
    let exe_plugins_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|parent| parent.join("plugins")));

    let (plugins_dir, enabled_list) = if let Some(ref dir) = exe_plugins_dir {
        if dir.exists() {
            // If plugins are present next to the executable, try to load enabled.json from there.
            let exe_root = dir.parent().unwrap_or_else(|| Path::new("."));
            let enabled = load_enabled_list(exe_root).unwrap_or_default();
            (dir.clone(), enabled)
        } else {
            let dir = repo_root.join("plugins");
            let enabled = load_enabled_list(repo_root).unwrap_or_default();
            (dir, enabled)
        }
    } else {
        let dir = repo_root.join("plugins");
        let enabled = load_enabled_list(repo_root).unwrap_or_default();
        (dir, enabled)
    };

    let mut entries = vec![];
    if !plugins_dir.exists() {
        return Ok(entries);
    }

    for entry in fs::read_dir(&plugins_dir)? {
        let ent = entry?;
        let path = ent.path();
        if path.is_dir() {
            let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
            let enabled = enabled_list.contains(&dir_name);

            // Attempt to read and validate plugin.toml
            let plugin_toml = path.join("plugin.toml");
            let metadata = if plugin_toml.exists() {
                match crate::plugins::validation::validate_manifest(&plugin_toml) {
                    Ok((m, _caps)) => Some(m),
                    Err(err) => {
                        eprintln!("warning: invalid plugin manifest {:?}: {}", plugin_toml, err);
                        None
                    }
                }
            } else {
                None
            };

            entries.push(PluginEntry { dir_name, path, enabled, metadata });
        }
    }

    Ok(entries)
}
