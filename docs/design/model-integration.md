<!--
Design doc: Model integration, adapters & plugins
Location: docs/design/model-integration.md
-->

# Model Integration & Adapter Architecture

目的

- 本設計書は、本リポジトリでのモデル・プロバイダ統合戦略（ローカル Ollama、将来の GPT / Copilot 等）と、モデルごとの呼出しフォーマット差分を扱うための "adapter/plugin" 方針を記録します。

要点（決定事項）

- デフォルトでサポートするモデル: `llama3.1:8b`（`model-adapter` の `Llama3DefaultAdapter`）
- モデル固有の function-calling/ツール呼出フォーマットは `ModelAdapter` に内包し、将来の増加はプラグインで拡張する。
- `Phi-4-mini-instruct` は外部プラグイン (`crates/plugins/phi4-mini-adapter`) で提供し、チャットテンプレート差分を runtime で差し替え可能にした。
- モデルへの送信/受信（HTTP 等）の責務は `ModelProvider`（例: `ollama-client`）が担当する。

背景と理由

- モデルは多数かつ増え続けるため、コアに全モデル固有ロジックを入れると肥大化し保守困難になる。
- 共通部分（接続・認証・health/generate の抽象化）は `model-provider` に集約し、モデル固有の入出力整形は `model-adapter`（プラグイン）で分離する方が拡張性・テスト性に優れる。

主要コンポーネント（現在の実装）

- `crates/model-provider`: Provider 抽象 (`ModelProvider` trait)、`ProviderError`、GenerateResult。
- `crates/ollama-client`: ローカル Ollama との HTTP 通信クライアント（`generate`, `health`）。
- `crates/model-adapter`: ModelAdapter trait と `Llama3DefaultAdapter`（`llama3.1:8b` を既定サポート）。

インターフェース概要

- ModelProvider (既存)
  - fn name(&self) -> &str
  - async fn health(&self) -> Result<bool, ProviderError>
  - async fn generate(&self, model: &str, prompt: &str) -> Result<GenerateResult, ProviderError>

- ModelAdapter (新規)
  - fn adapter_name(&self) -> &str
  - fn supported_models(&self) -> Vec`<String>`
  - async fn invoke(&self, provider: &dyn ModelProvider, model: &str, prompt: &str, tools: Option<&[ToolSpec]>) -> Result<GenerateResult, ProviderError>

ToolSpec

- モデルが呼び出し可能なツール（関数）を表現する軽量構造体。`name`, `description`, `schema` を持つ。

運用フロー（実行時）

1. アプリケーションは利用可能な adapter を列挙（組み込み + plugins/ からのロード）し、ユーザに選択肢を提示する。
2. 選択された model name（例: `llama3.1:8b`）に対応する adapter を決定。
3. アダプタは与えられた `ToolSpec` 等をモデル固有形式に直列化し、`ModelProvider::generate` を呼ぶ。
4. 返却された文字列を adapter が必要に応じて解析（JSON ⇒ structured field）して `GenerateResult` を返す。

## UI / Controller 連携（2025-12-03 更新）

- `chat-core::ChatController` が `ChatState::available_models` を保持し、`ChatCommand::RefreshModels` 実行時に `OllamaClient` で一覧を取得する。
- モデル検出完了時は `ChatEvent::ModelsUpdated` を発火し、UI 側は共有 `ChatState` スナップショットだけを参照して `neko-ui::model_selector_row` を更新する。
- UI からのモデル切替は `ChatCommand::SwitchModel` に集約され、成功時のみ `ChatEvent::ModelChanged` で同期される。これにより UI は独自の async runtime を持たず、`ChatController` が唯一の I/O 境界となる。
- 既定モデルや curated リストは controller が管理し、フォールバックも内部で処理するため、UI は単に `available_models` を props としてレンダリングすればよい。

プラグイン戦略

- 新しいモデルフォーマット対応は `plugins/` (将来的に) または `crates/plugins/<adapter>`（当面はワークスペース内 crate 追加）として公開。
  - 例: `crates/plugins/phi4-mini-adapter` が Phi-4 用 Formatter を配布中。
- プラグインは `ModelAdapter` を実装すれば良い。外部開発者向けに `crates/model-adapter/templates` を用意することを推奨。

設定例（TOML）

```toml
[model]
default = "llama3.1:8b"
adapter_dirs = ["plugins/adapters"]
```

テストと CI

- Adapter 単位でユニットテストを用意（モック ModelProvider を注入）。
- CI では `cargo test -p model-adapter` を実行。`ollama-impl` のインテグレーションテストは実環境依存のため分離し、手動/インテグレーション環境で実行。

ドキュメント配置方針

- 設計書はルート `docs/design/` に格納する。各クレートにも `README.md` を置いて実装・使用方法を示す（既存の慣習）。
- ここに残すべき追加ドキュメント:
  - `model-integration.md`（本ファイル）
  - `plugins.md`（プラグイン作成手順テンプレ）
  - `provider-contracts.md`（Provider/Adapter の詳細型定義）

今後の作業候補（優先度）

1. `plugins/adapter-template` を追加して外部向けテンプレートを用意。
2. `neko-assistant` 側にモデル選択 UI と adapter 管理を実装。
3. OpenAI / Copilot アダプタの設計・プロトタイプ作成。

付録: 既知モデルリスト（参考）

- okamototk/llama-swallow:8b
- llama3.1:8b (default)
- gemma3:4b
- qwen3:8b
- phi4-mini:latest
- pakachan/elyza-llama3-8b:latest

---
このファイルをベースに、プラグインテンプレートや provider 契約の詳細を追記していきます。必要なら私が `plugins/adapter-template` と `docs/design/plugins.md` を作成します。どちらを先に作りますか？
