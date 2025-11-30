use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

const ENABLED_FILE: &str = "plugins/enabled.json";

pub fn load_enabled_list(repo_root: &Path) -> io::Result<Vec<String>> {
    let f = repo_root.join(ENABLED_FILE);
    if !f.exists() {
        return Ok(vec![]);
    }
    let s = fs::read_to_string(f)?;
    let v: Vec<String> = serde_json::from_str(&s).unwrap_or_default();
    Ok(v)
}

pub fn save_enabled_list(repo_root: &Path, list: &[String]) -> io::Result<()> {
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
