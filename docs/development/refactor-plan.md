# ChatView リファクタ計画

以下の計画は、neko-assistant のチャット機能を段階的に分離・再構成し、ロジックと UI の責務を明確に分けることを目的とします。すべてのフェーズは順番に進め、完了したらチェックボックスへチェックを入れてください。

## Phase 1: Chat コア抽出
- [x] `ConversationService` / `MessageHandler` 周辺のロジックを独立クレート（仮称 `chat-core`）へ移動する。
- [x] `ChatController`（会話ID、アクティブモデル、メッセージ配列等を保持）を作成し、UI とは `ChatCommand` / `ChatEvent` で通信する API を定義する。
- [x] ロジック層から UI への副作用はコールバック（例: `on_messages_updated`, `on_model_changed`）で通知するようにする。
- [x] `chat-core` にユニットテストを追加し、`cargo test -p chat-core` が成立する状態を作る。

## Phase 2: UI クレート整理
- [x] `neko-ui` に `ChatSidebar`, `ModelSelectorRow`, `ScratchpadConsole` などの純 UI コンポーネントを移管し、`gpui` 以外の依存を持たないようにする。
- [x] `neko-assistant` 側の `ChatView` は `chat-core::ChatController` と `neko-ui` コンポーネントを組み合わせるだけの薄いレイヤにする（2025-12-04、`ChatUiSnapshot` で状態整形を集約）。
	- [x] チャットメッセージ一覧／入力パネル／ツールバーを `neko-ui` コンポーネント化（2025-12-03）
	- [x] メッセージスクロールパネルを `neko-ui::chat_messages_panel` として分離（2025-12-03）
	- [x] メインペイン／ワークスペース全体を `chat_main_panel` / `chat_workspace` に集約（2025-12-03）
	- [x] `ChatView` ツールバーから `StyledExt` 依存を除去し、gpui 標準 API へ統一（2025-12-03）
- [x] UI で保持している状態（入力、スクロール等）を `ChatController` の state から props へ変換する小さなマッパー関数を用意する。

## Phase 3: イベント／状態の明文化
- [x] `ChatState` を `Arc<RwLock<_>>` もしくは `tokio::sync::watch` 等で公開し、UI 側は `Subscription` 経由で購読できるようにする（2025-12-04、`watch::channel` でスナップショット配信）。
	- [x] `chat-core::ChatController` の状態共有を `Arc<RwLock<ChatState>>` に変更し、`ChatEvent::StateChanged` は通知のみ（2025-12-03）
	- [x] 会話メタデータも `ChatState` に含め、`ChatEvent::ConversationsUpdated` も通知のみへ変更（2025-12-03）
	- [x] `ChatEvent::ModelChanged` も通知のみとし、UI は共有状態からアクティブモデルを取得（2025-12-03）
	- [x] `ChatState` に MCP サーバー／ツールのメタデータを追加し、`RefreshMcpMetadata` コマンドで更新（2025-12-03）
	- [x] モデル検出を `ChatController::refresh_available_models` に集約し、`ChatEvent::ModelsUpdated`/`ChatState::available_models` で UI を駆動（2025-12-03）
- [x] UI から非同期処理（モデル呼び出し、MCP 呼び出し等）を直接行わず、必ず `ChatController` を経由するようにコードベースを検索・修正する（2025-12-03、`neko-assistant/src/gui` から `tokio` / `OllamaClient` 依存を排除）。
	- [x] ChatView から Ollama モデル探索スレッドを撤去し、全てのモデル更新を `ChatCommand::RefreshModels` 経由に統一（2025-12-03）
- [x] `ConversationActions` を `chat-core` に統合し、UI 層には「イベントを送るだけ」のインターフェイスを残す（2025-12-03、旧 helper 削除済み）。

## Phase 4: 仕上げ・ドキュメント
- [x] すべてのフェーズ完了後に `docs/development/phase5-mcp-integration.md` など関連ドキュメントを更新する（2025-12-04、MCP/状態配信の記述を追記）。
- [x] `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace`, `cargo run -p neko-assistant -- test-mcp` のチェックリストを PR 説明に追記する（2025-12-04、coding-guidelines に明示）。
- [x] 主要変更点（新クレート追加や API 変更）を `CHANGELOG.md` に追記する（2025-12-04）。

## Phase 5: クレート棚卸し
- [ ] 新しい構成が安定したら、`crates/` 配下の全クレートと `neko-assistant` 直下のモジュールを棚卸しし、未使用・重複・責務が重なっているものを洗い出す。
- [x] `cargo metadata --format-version 1 > target/metadata.json` を実行して現行ワークスペースのクレート一覧を固定化する（2025-12-05、`target/metadata.json` 保存済み）。
- [x] `cargo tree --workspace --no-default-features > target/cargo-tree-no-default.txt` で依存グラフを確認し、feature 依存の重複を洗い出す（2025-12-05）。
- [x] `cargo +nightly udeps --workspace > target/cargo-udeps.txt` 導入可否を検討し、未使用依存の検出ログを残す（2025-12-05、`mcp-weather-server` の `thiserror` と `neko-ui` の `ui-utils` が未使用候補）。
- [ ] 使われなくなったクレート／モジュールの削除 PR を作成し、依存関係から除去する。
- [ ] まだ必要だが責務が曖昧なクレートには TODO コメントや ISSUE を残し、次のリファクタ対象を明確にする。

> 進行中のフェーズやタスクが完了したら、該当チェックボックスにチェック（`[x]`）を入れて常に最新状態を保ってください。
