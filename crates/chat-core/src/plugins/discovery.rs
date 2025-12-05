use crate::plugins::metadata::PluginEntry;
use std::fs;
use std::io;
use std::path::Path;

/// Discover plugin directories under the repository `plugins/` directory.
pub fn discover_plugins(repo_root: &Path) -> io::Result<Vec<PluginEntry>> {
    let exe_plugins_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|parent| parent.join("plugins")));

    let plugins_dir = if let Some(ref dir) = exe_plugins_dir {
        if dir.exists() {
            dir.clone()
        } else {
            repo_root.join("plugins")
        }
    } else {
        repo_root.join("plugins")
    };

    let mut entries = vec![];
    if !plugins_dir.exists() {
        return Ok(entries);
    }

    for entry in fs::read_dir(&plugins_dir)? {
        let ent = entry?;
        let path = ent.path();
        if path.is_dir() {
            let dir_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            // Determine whether the plugin should be treated as enabled.
            // Historically we used an `enabled.json` file, but that
            // management step is unnecessary: load a plugin when a
            // platform shared library exists in the plugin directory.
            let enabled = plugin_has_shared_library(&path);

            let plugin_toml = path.join("plugin.toml");
            let metadata = if plugin_toml.exists() {
                match crate::plugins::validation::validate_manifest(&plugin_toml) {
                    Ok((m, _caps)) => Some(m),
                    Err(err) => {
                        eprintln!(
                            "warning: invalid plugin manifest {:?}: {}",
                            plugin_toml, err
                        );
                        None
                    }
                }
            } else {
                None
            };

            entries.push(PluginEntry {
                dir_name,
                path,
                enabled,
                metadata,
            });
        }
    }

    Ok(entries)
}

/// Check plugin directory for a platform dynamic library (dll/so/dylib).
fn plugin_has_shared_library(path: &Path) -> bool {
    if let Ok(rd) = fs::read_dir(path) {
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_file() {
                if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                    match ext.to_lowercase().as_str() {
                        "dll" | "so" | "dylib" => return true,
                        _ => {}
                    }
                }
            }
        }
    }
    false
}
