use crate::plugins::metadata::{PluginEntry, PluginMetadata};
use crate::plugins::enabled::load_enabled_list;
use std::fs;
use std::io;
use std::path::Path;

/// Discover plugin directories under the repository `plugins/` directory.
/// If a `plugin.toml` is present in each plugin dir it will be parsed and returned as metadata.
pub fn discover_plugins(repo_root: &Path) -> io::Result<Vec<PluginEntry>> {
    let plugins_dir = repo_root.join("plugins");
    let mut entries = vec![];

    // Load enabled list if present
    let enabled_list = load_enabled_list(repo_root).unwrap_or_default();

    if !plugins_dir.exists() {
        return Ok(entries);
    }

    for entry in fs::read_dir(&plugins_dir)? {
        let ent = entry?;
        let path = ent.path();
        if path.is_dir() {
            let dir_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
            let enabled = enabled_list.contains(&dir_name);

            // Attempt to read plugin.toml
            let plugin_toml = path.join("plugin.toml");
            let metadata = if plugin_toml.exists() {
                match fs::read_to_string(&plugin_toml) {
                    Ok(s) => match toml::from_str::<PluginMetadata>(&s) {
                        Ok(m) => Some(m),
                        Err(_) => None,
                    },
                    Err(_) => None,
                }
            } else {
                None
            };

            entries.push(PluginEntry { dir_name, path, enabled, metadata });
        }
    }

    Ok(entries)
}
