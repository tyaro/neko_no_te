/// アプリケーション設定管理
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// アプリケーション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// OllamaのベースURL
    pub ollama_base_url: String,
    /// デフォルトモデル
    pub default_model: String,
    /// 最大履歴メッセージ数
    pub max_history_messages: usize,
    /// LangChainを使用するかどうか
    pub use_langchain: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ollama_base_url: "http://localhost:11434".to_string(),
            default_model: "phi4-mini:3.8b".to_string(),
            max_history_messages: 20,
            use_langchain: false, // デフォルトは既存実装
        }
    }
}

impl AppConfig {
    /// 設定ファイルのパスを取得
    fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("neko-assistant");
        
        std::fs::create_dir_all(&config_dir).ok();
        config_dir.join("config.toml")
    }

    /// 設定を読み込む（存在しない場合はデフォルト値）
    pub fn load_or_default() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => eprintln!("Failed to parse config: {}", e),
                },
                Err(e) => eprintln!("Failed to read config: {}", e),
            }
        }
        Self::default()
    }

    /// 設定を保存
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        println!("Config saved to: {}", path.display());
        Ok(())
    }
}
