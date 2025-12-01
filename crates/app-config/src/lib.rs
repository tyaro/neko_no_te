//! アプリケーション設定管理
//!
//! このクレートは neko-assistant の設定を管理します。
//! - デフォルト設定の提供
//! - TOML ファイルからの読み込み
//! - 設定の保存

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// アプリケーション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Ollama ベース URL
    #[serde(default = "default_ollama_url")]
    pub ollama_base_url: String,
    
    /// デフォルトモデル名
    #[serde(default = "default_model")]
    pub default_model: String,
    
    /// 最大メッセージ履歴数（メモリ内保持）
    #[serde(default = "default_max_history")]
    pub max_history_messages: usize,
    
    /// セッション保存ディレクトリ
    #[serde(default = "default_session_dir")]
    pub session_dir: PathBuf,
    
    /// 送信キー設定 ("enter" または "ctrl_enter")
    #[serde(default = "default_send_key")]
    pub send_key: String,
    
    /// LangChain を使用するかどうか
    #[serde(default = "default_use_langchain")]
    pub use_langchain: bool,
}

fn default_send_key() -> String {
    "ctrl_enter".to_string()
}

fn default_use_langchain() -> bool {
    false // デフォルトは既存の実装を使用
}

fn default_ollama_url() -> String {
    "http://localhost:11434/".to_string()
}

fn default_model() -> String {
    "phi4-mini:3.8b".to_string()
}

fn default_max_history() -> usize {
    100
}

fn default_session_dir() -> PathBuf {
    get_default_data_dir().join("sessions")
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ollama_base_url: default_ollama_url(),
            default_model: default_model(),
            max_history_messages: default_max_history(),
            session_dir: default_session_dir(),
            send_key: default_send_key(),
            use_langchain: default_use_langchain(),
        }
    }
}

impl AppConfig {
    /// デフォルト設定を作成
    pub fn new() -> Self {
        Self::default()
    }
    
    /// TOML ファイルから設定を読み込み
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config: AppConfig = toml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;
        
        Ok(config)
    }
    
    /// デフォルト設定ファイルパスから読み込み（なければデフォルト設定を返す）
    pub fn load_or_default() -> Self {
        let config_path = get_default_config_path();
        
        if config_path.exists() {
            Self::load_from_file(&config_path).unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load config ({}), using defaults", e);
                Self::default()
            })
        } else {
            Self::default()
        }
    }
    
    /// 設定をファイルに保存
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        // 親ディレクトリを作成
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }
        
        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
        
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        
        Ok(())
    }
    
    /// デフォルト設定ファイルパスに保存
    pub fn save(&self) -> Result<()> {
        let config_path = get_default_config_path();
        self.save_to_file(&config_path)
    }
}

/// デフォルトのデータディレクトリを取得
/// Windows: %USERPROFILE%\.neko-assistant
/// Unix: ~/.neko-assistant
pub fn get_default_data_dir() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".neko-assistant")
    } else {
        PathBuf::from(".neko-assistant")
    }
}

/// デフォルトの設定ファイルパスを取得
pub fn get_default_config_path() -> PathBuf {
    get_default_data_dir().join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.ollama_base_url, "http://localhost:11434/");
        assert_eq!(config.default_model, "phi4-mini:3.8b");
        assert_eq!(config.max_history_messages, 100);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let config = AppConfig {
            ollama_base_url: "http://custom:8080/".to_string(),
            default_model: "custom-model".to_string(),
            max_history_messages: 50,
            session_dir: PathBuf::from("/tmp/sessions"),
            send_key: "ctrl_enter".to_string(),
            use_langchain: false,
        };
        
        // 保存
        config.save_to_file(&config_path).unwrap();
        
        // 読み込み
        let loaded = AppConfig::load_from_file(&config_path).unwrap();
        assert_eq!(loaded.ollama_base_url, config.ollama_base_url);
        assert_eq!(loaded.default_model, config.default_model);
        assert_eq!(loaded.max_history_messages, config.max_history_messages);
    }

    #[test]
    fn test_load_or_default() {
        // 存在しないファイルの場合はデフォルトを返す
        let config = AppConfig::load_or_default();
        assert!(!config.ollama_base_url.is_empty());
    }
}
