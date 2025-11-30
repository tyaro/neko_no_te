use crate::plugins::metadata::PluginMetadata;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Read and parse a plugin TOML manifest file into a toml::Value
pub fn read_manifest(path: &Path) -> Result<toml::Value> {
    let s = fs::read_to_string(path).with_context(|| format!("reading manifest {:?}", path))?;
    let v = toml::from_str::<toml::Value>(&s).with_context(|| "parsing toml manifest")?;
    Ok(v)
}

/// Extract the simple PluginMetadata used by the rest of the codebase.
pub fn extract_metadata(value: &toml::Value) -> PluginMetadata {
    let name = value.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let description = value.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let version = value.get("version").and_then(|v| v.as_str()).map(|s| s.to_string());
    let author = value.get("author").and_then(|v| v.as_str()).map(|s| s.to_string());
    PluginMetadata { name, description, version, author }
}

/// Extract declared capabilities as a set of keys where the value is truthy.
pub fn extract_capabilities(value: &toml::Value) -> HashSet<String> {
    let mut set = HashSet::new();
    if let Some(caps) = value.get("capabilities") {
        if let Some(tbl) = caps.as_table() {
            for (k, v) in tbl.iter() {
                if v.as_bool().unwrap_or(false) {
                    set.insert(k.clone());
                }
            }
        }
    }
    set
}

/// Validate a manifest file at the given path. Returns extracted metadata and capability set.
pub fn validate_manifest(path: &Path) -> Result<(PluginMetadata, HashSet<String>)> {
    let v = read_manifest(path)?;

    // Basic validation: if name exists it must be non-empty. Version if present should be non-empty.
    if let Some(name) = v.get("name").and_then(|x| x.as_str()) {
        if name.trim().is_empty() {
            anyhow::bail!("manifest 'name' is empty");
        }
    }

    let metadata = extract_metadata(&v);
    let caps = extract_capabilities(&v);
    Ok((metadata, caps))
}
