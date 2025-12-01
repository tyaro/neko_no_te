use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
