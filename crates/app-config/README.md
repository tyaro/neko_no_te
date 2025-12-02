# app-config

neko-assistant のアプリケーション設定管理クレート。

## 機能

- デフォルト設定の提供
- SQLite による設定永続化（アプリ実行ファイルと同階層に `neko_assistant_settings.db` を作成）
- 既存 TOML (`~/.neko-assistant/config.toml`) からの自動マイグレーション

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

## 保存場所

- アプリの実行ファイルと同じディレクトリに `neko_assistant_settings.db` を作成します。
- 以前の `config.toml`（`~/.neko-assistant/config.toml`）が存在する場合は初回起動時に読み込み、内容を SQLite に移行します。

TOML のバックアップが必要な場合は、従来どおり `AppConfig::save_to_file` / `load_from_file` を利用してください。
