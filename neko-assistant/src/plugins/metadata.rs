use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub kind: PluginKind,
    pub entrypoint: Option<String>,
    pub library: Option<String>,
    pub models: Vec<String>,
    pub priority: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    Adapter,
    PromptBuilder,
    #[serde(other)]
    Other,
}

impl Default for PluginKind {
    fn default() -> Self {
        PluginKind::Adapter
    }
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
