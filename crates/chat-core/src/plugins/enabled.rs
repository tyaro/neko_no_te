use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

const ENABLED_FILE: &str = "enabled.json";

/// Load enabled plugins list from repo_root or exe directory
pub fn load_enabled_list(repo_root: &Path) -> io::Result<Vec<String>> {
    // Try exe directory first (for deployed binaries)
    let exe_enabled = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|parent| parent.join(ENABLED_FILE)));

    let f = if let Some(ref path) = exe_enabled {
        if path.exists() {
            path.clone()
        } else {
            repo_root.join("plugins").join(ENABLED_FILE)
        }
    } else {
        repo_root.join("plugins").join(ENABLED_FILE)
    };

    if !f.exists() {
        return Ok(vec![]);
    }
    let s = fs::read_to_string(f)?;
    let v: Vec<String> = serde_json::from_str(&s).unwrap_or_default();
    Ok(v)
}

/// Save enabled plugins list to repo_root or exe directory
pub fn save_enabled_list(repo_root: &Path, list: &[String]) -> io::Result<()> {
    // Try exe directory first (for deployed binaries)
    let exe_enabled = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|parent| parent.join(ENABLED_FILE)));

    let f = if let Some(ref path) = exe_enabled {
        // If exe directory is writable, use it
        if let Ok(metadata) = fs::metadata(path.parent().unwrap_or_else(|| Path::new("."))) {
            if !metadata.permissions().readonly() {
                path.clone()
            } else {
                repo_root.join("plugins").join(ENABLED_FILE)
            }
        } else {
            path.clone()
        }
    } else {
        repo_root.join("plugins").join(ENABLED_FILE)
    };

    if let Some(parent) = f.parent() {
        fs::create_dir_all(parent)?;
    }
    let s = serde_json::to_string_pretty(list)?;
    let mut file = fs::File::create(f)?;
    file.write_all(s.as_bytes())?;
    Ok(())
}

pub fn enable_plugin(repo_root: &Path, plugin_name: &str) -> io::Result<()> {
    let mut list = load_enabled_list(repo_root).unwrap_or_default();
    if !list.contains(&plugin_name.to_string()) {
        list.push(plugin_name.to_string());
    }
    save_enabled_list(repo_root, &list)
}

pub fn disable_plugin(repo_root: &Path, plugin_name: &str) -> io::Result<()> {
    let mut list = load_enabled_list(repo_root).unwrap_or_default();
    list.retain(|n| n != plugin_name);
    save_enabled_list(repo_root, &list)
}
