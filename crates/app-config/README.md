# app-config

neko-assistant のアプリケーション設定管理クレート。

## 機能

- デフォルト設定の提供
- TOML 形式での設定ファイルの読み書き
- プラットフォーム別のデフォルトパス管理

## 使用例

```rust
use app_config::AppConfig;

// デフォルト設定を使用
let config = AppConfig::default();

// 設定ファイルから読み込み（なければデフォルト）
let config = AppConfig::load_or_default();

// 設定を変更して保存
let mut config = AppConfig::default();
config.default_model = "custom-model".to_string();
config.save().unwrap();
```

## 設定項目

| 項目 | デフォルト値 | 説明 |
|------|-------------|------|
| `ollama_base_url` | `http://localhost:11434/` | Ollama API のベース URL |
| `default_model` | `phi4-mini:3.8b` | デフォルトで使用するモデル |
| `max_history_messages` | `100` | メモリ内で保持する最大メッセージ数 |
| `session_dir` | `~/.neko-assistant/sessions` | セッション保存ディレクトリ |

## 設定ファイルの場所

- **Windows**: `%USERPROFILE%\.neko-assistant\config.toml`
- **Unix/Linux/macOS**: `~/.neko-assistant/config.toml`

## 設定ファイル例

```toml
ollama_base_url = "http://localhost:11434/"
default_model = "phi4-mini:3.8b"
max_history_messages = 100
session_dir = "/home/user/.neko-assistant/sessions"
```
