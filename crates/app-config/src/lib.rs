//! アプリケーション設定管理
//!
//! このクレートは neko-assistant の設定を管理します。
//! - デフォルト設定の提供
//! - TOML ファイルからの読み込み
//! - 設定の保存

use anyhow::{anyhow, Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DB_FILE_NAME: &str = "neko_assistant_settings.db";

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

        let config: AppConfig =
            toml::from_str(&content).with_context(|| "Failed to parse config file")?;

        Ok(config)
    }

    /// SQLite から設定を読み込み（存在しない場合は None）
    pub fn load_from_database(path: &Path) -> Result<Option<Self>> {
        let conn = open_database(path)?;
        let mut stmt = conn
            .prepare(
                "SELECT ollama_base_url, default_model, max_history_messages, session_dir, send_key, use_langchain
                 FROM app_config
                 WHERE id = 1",
            )
            .context("Failed to prepare config query")?;

        let result = stmt.query_row([], |row| {
            let max_history: i64 = row.get(2)?;
            let session_dir: String = row.get(3)?;
            let use_langchain_raw: i64 = row.get(5)?;
            let max_history_messages = max_history.try_into().unwrap_or(0);

            Ok(AppConfig {
                ollama_base_url: row.get(0)?,
                default_model: row.get(1)?,
                max_history_messages,
                session_dir: PathBuf::from(session_dir),
                send_key: row.get(4)?,
                use_langchain: use_langchain_raw != 0,
            })
        });

        match result {
            Ok(config) => Ok(Some(config)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// デフォルト DB パスから読み込み（なければデフォルト設定を返す）
    pub fn load_or_default() -> Self {
        match Self::load_from_default_database() {
            Ok(Some(config)) => return config,
            Ok(None) => {}
            Err(e) => eprintln!("Warning: Failed to read SQLite config ({}).", e),
        }

        if let Some(legacy) = load_legacy_config_file() {
            if let Err(e) = legacy.save() {
                eprintln!("Warning: Failed to migrate config to SQLite ({}).", e);
            }
            return legacy;
        }

        let config = Self::default();
        if let Err(e) = config.save() {
            eprintln!("Warning: Failed to persist default config ({}).", e);
        }
        config
    }

    /// 設定をファイルに保存
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        // 親ディレクトリを作成
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let content = toml::to_string_pretty(self).with_context(|| "Failed to serialize config")?;

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    /// SQLite データベースに保存
    pub fn save_to_database(&self, path: &Path) -> Result<()> {
        let conn = open_database(path)?;
        let session_dir = self.session_dir.to_string_lossy().to_string();
        let max_history: i64 = self
            .max_history_messages
            .try_into()
            .map_err(|_| anyhow!("max_history_messages exceeds supported range"))?;

        conn.execute(
            "INSERT INTO app_config (id, ollama_base_url, default_model, max_history_messages, session_dir, send_key, use_langchain)
             VALUES (1, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                 ollama_base_url = excluded.ollama_base_url,
                 default_model = excluded.default_model,
                 max_history_messages = excluded.max_history_messages,
                 session_dir = excluded.session_dir,
                 send_key = excluded.send_key,
                 use_langchain = excluded.use_langchain",
            params![
                &self.ollama_base_url,
                &self.default_model,
                max_history,
                session_dir,
                &self.send_key,
                if self.use_langchain { 1 } else { 0 }
            ],
        )
        .context("Failed to persist app_config row")?;

        Ok(())
    }

    /// デフォルト DB パスに保存
    pub fn save(&self) -> Result<()> {
        let db_path = default_db_path()?;
        self.save_to_database(&db_path)
    }

    fn load_from_default_database() -> Result<Option<Self>> {
        let path = default_db_path()?;
        Self::load_from_database(&path)
    }
}

fn load_legacy_config_file() -> Option<AppConfig> {
    let legacy_path = get_default_config_path();
    if !legacy_path.exists() {
        return None;
    }

    match AppConfig::load_from_file(&legacy_path) {
        Ok(config) => Some(config),
        Err(e) => {
            eprintln!(
                "Warning: Failed to load legacy config file ({}), falling back to defaults.",
                e
            );
            None
        }
    }
}

fn default_db_path() -> Result<PathBuf> {
    let exe = env::current_exe().context("Failed to determine executable path")?;
    let dir = exe
        .parent()
        .ok_or_else(|| anyhow!("Executable path has no parent directory"))?;
    Ok(dir.join(DB_FILE_NAME))
}

fn open_database(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create database directory: {}", parent.display())
        })?;
    }

    let conn = Connection::open(path)
        .with_context(|| format!("Failed to open SQLite database: {}", path.display()))?;
    ensure_schema(&conn)?;
    Ok(conn)
}

fn ensure_schema(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            ollama_base_url TEXT NOT NULL,
            default_model TEXT NOT NULL,
            max_history_messages INTEGER NOT NULL,
            session_dir TEXT NOT NULL,
            send_key TEXT NOT NULL,
            use_langchain INTEGER NOT NULL
        )",
        [],
    )
    .context("Failed to create app_config table")?;

    Ok(())
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
        let db_path = temp_dir.path().join("settings.db");

        let config = AppConfig {
            ollama_base_url: "http://custom:8080/".to_string(),
            default_model: "custom-model".to_string(),
            max_history_messages: 50,
            session_dir: PathBuf::from("/tmp/sessions"),
            send_key: "ctrl_enter".to_string(),
            use_langchain: false,
        };

        // 保存
        config.save_to_database(&db_path).unwrap();

        // 読み込み
        let loaded = AppConfig::load_from_database(&db_path)
            .unwrap()
            .expect("config row should exist");
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
