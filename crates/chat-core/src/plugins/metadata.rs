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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    #[default]
    Adapter,
    PromptBuilder,
    #[serde(other)]
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginEntry {
    pub dir_name: String,
    pub path: PathBuf,
    pub enabled: bool,
    pub metadata: Option<PluginMetadata>,
}
