use crate::plugins::metadata::{PluginKind, PluginMetadata};
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn read_manifest(path: &Path) -> Result<toml::Value> {
    let s = fs::read_to_string(path).with_context(|| format!("reading manifest {:?}", path))?;
    let v = toml::from_str::<toml::Value>(&s).with_context(|| "parsing toml manifest")?;
    Ok(v)
}

pub fn extract_metadata(value: &toml::Value) -> PluginMetadata {
    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let description = value
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let version = value
        .get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let author = value
        .get("author")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let kind = value
        .get("kind")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "prompt_builder" => PluginKind::PromptBuilder,
            "adapter" | _ => PluginKind::Adapter,
        })
        .unwrap_or_default();

    let entrypoint = value
        .get("entrypoint")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let library = value
        .get("library")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let models = value
        .get("models")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let priority = value
        .get("priority")
        .and_then(|v| v.as_integer())
        .map(|v| v as i32);

    PluginMetadata {
        name,
        description,
        version,
        author,
        kind,
        entrypoint,
        library,
        models,
        priority,
    }
}

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

pub fn validate_manifest(path: &Path) -> Result<(PluginMetadata, HashSet<String>)> {
    let v = read_manifest(path)?;

    if let Some(name) = v.get("name").and_then(|x| x.as_str()) {
        if name.trim().is_empty() {
            anyhow::bail!("manifest 'name' is empty");
        }
    }

    let metadata = extract_metadata(&v);

    if matches!(metadata.kind, PluginKind::PromptBuilder) {
        if metadata
            .entrypoint
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
        {
            anyhow::bail!("prompt_builder requires non-empty 'entrypoint'");
        }
        if metadata.library.as_deref().unwrap_or("").trim().is_empty() {
            anyhow::bail!("prompt_builder requires non-empty 'library'");
        }
        if metadata.models.is_empty() {
            anyhow::bail!("prompt_builder requires at least one model in 'models'");
        }
    }
    let caps = extract_capabilities(&v);
    Ok((metadata, caps))
}
