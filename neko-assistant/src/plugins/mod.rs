use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginEntry {
    /// Directory name under `plugins/`
    pub dir_name: String,
    /// Path to the plugin directory
    pub path: PathBuf,
    /// Whether plugin is enabled (from plugins/enabled.json)
    pub enabled: bool,
    /// Optional metadata parsed from `plugin.toml`
    pub metadata: Option<PluginMetadata>,
}

const ENABLED_FILE: &str = "plugins/enabled.json";

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

fn load_enabled_list(repo_root: &Path) -> io::Result<Vec<String>> {
    let f = repo_root.join(ENABLED_FILE);
    if !f.exists() {
        return Ok(vec![]);
    }
    let s = fs::read_to_string(f)?;
    let v: Vec<String> = serde_json::from_str(&s).unwrap_or_default();
    Ok(v)
}

fn save_enabled_list(repo_root: &Path, list: &[String]) -> io::Result<()> {
    let f = repo_root.join(ENABLED_FILE);
    if let Some(parent) = f.parent() {
        fs::create_dir_all(parent)?;
    }
    let s = serde_json::to_string_pretty(list)?;
    let mut file = fs::File::create(f)?;
    file.write_all(s.as_bytes())?;
    Ok(())
}

/// Enable a plugin by name (adds to enabled.json)
pub fn enable_plugin(repo_root: &Path, plugin_name: &str) -> io::Result<()> {
    let mut list = load_enabled_list(repo_root).unwrap_or_default();
    if !list.contains(&plugin_name.to_string()) {
        list.push(plugin_name.to_string());
    }
    save_enabled_list(repo_root, &list)
}

/// Disable a plugin by name (removes from enabled.json)
pub fn disable_plugin(repo_root: &Path, plugin_name: &str) -> io::Result<()> {
    let mut list = load_enabled_list(repo_root).unwrap_or_default();
    list.retain(|n| n != plugin_name);
    save_enabled_list(repo_root, &list)
}
