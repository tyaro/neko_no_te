# Copilot Instructions for neko_no_te
- **言語**: すべて日本語で記述。最初に `docs/development/coding-guidelines.md` を読み、小分けモジュール指針と PR 作法を把握。
- **全体像**: Rust ワークスペース。`neko-assistant/` が GPUI フロント、`crates/` が Provider/Adapter/LLM 実装、`plugins/` が実行時ロードツール、`research/` が検証用 Python サンドボックス。
- **主要境界**: `model-provider` が HTTP/認証、`model-adapter` が function-calling 整形、`langchain-bridge`/`chat-engine` がチェーン制御、`chat-history` が永続化、`ui-utils`+`neko-ui` が再利用 UI。
- **会話フロー**: `MessageHandler` を唯一の UI↔LLM 窓口に保ち、Phase 5 の `ChatEvent` 方針（UserMessageReceived→ToolCallRequested→ToolResultReceived）を崩さない。UI から tokio::spawn を直接呼ばない。
- **LangChain 統合**: `langchain-bridge` がプロンプト構築をカプセル化。ツール記述は `McpManager::get_tools_description()` で取得し、システムプロンプトへ注入する想定。
- **MCP 統合**: `mcp_client.rs` が 1 サーバー JSON-RPC を担当し、`mcp_manager.rs` が `Arc<Mutex<HashMap<String, McpClient>>>` で複数管理。設定は 実行ファイルと同じディレクトリ（`cargo run` 時は `target/debug/mcp_servers.json`）を `load_mcp_config()` で読む。
- **検証コマンド**: `cargo run -p neko-assistant -- test-mcp` で initialize/list_tools/call_tool を一括確認。新サーバーを追加したらまずここで動作検証。
- **既知課題**: `McpClient::receive_response` が `BufReader::read_line` 依存でハングする。パッチでは `tokio::time::timeout(Duration::from_secs(10))` と JSON フレーミング、詳細 `eprintln!` を必ず入れる。
- **プラグイン開発**: `crates/plugins/adapter-template/` をコピー→`Cargo.toml`/`plugin.toml` を更新→`ModelAdapter::{supported_models, invoke}` を実装→`cargo test -p <plugin>`→`pwsh .\scripts\sync-plugins.ps1 -Configuration Debug` で `target/<config>/plugins/` に配置。
- **動的ロード**: `neko-assistant/src/plugins/{metadata,discovery,validation,enabled}.rs` が `plugin.toml` をパースし UI へ公開。シリアライズ項目を変えたら discovery/validation 両方を更新。
- **GPUI ルール**: `render()` では `try_lock()` で即クローンし、`cx.notify()` は通知チャネルと二重で呼ばない。スクロールは `flex_1 + h_full` の親→`overflow_hidden`→`overflow_y_scroll().track_scroll(handle)` の子で構成。
- **状態管理**: `chat-history` クレートで会話を取得/保存し、UI 側は immut な `ConversationSnapshot` を利用。ロック保持時間を最小化する。
- **ビルド/検証**: `cargo build --workspace`, `cargo test --workspace`, `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings` が PR 必須。LangChain/MCP 実験は `research/<feature>/` で venv を切り、本体に Python 依存を持ち込まない。
- **スクリプト**: `scripts/sync-plugins.ps1` が build 成果物→`target/*/plugins/` 同期を行う。`scripts/build_and_sync.ps1` はアプリビルドとプラグイン配置をまとめて行うため、GUI デバッグ前に実行。
- **ドキュメントの責務**: 設計は `docs/design/model-integration.md` と `docs/design/plugins.md`、UI 規約は `docs/development/phase3-lessons-learned.md`、Phase5 要件は `docs/development/phase5-mcp-integration.md`、LangChain 評価は `docs/research/langchain-rust-evaluation.md` を更新する。
- **テスト/トラブル**: プラグイン未検出→同期漏れを疑い `target/debug/plugins` を確認。`npx` が見つからない→Windows では `npx.cmd` を明示し PATH を継承する。
- **コミット慣習**: 1責務=1コミット、`type(scope): summary` 形式。コード変更時は対応する `docs/` か `TESTS.md` を更新し、`zed-fork/` ディレクトリは読み取り専用。
- **参考実装**: `neko-assistant/src/mcp_client.rs`, `mcp_manager.rs`, `message_handler.rs`, `plugins/mod.rs`, `docs/development/phase5-mcp-integration.md` を常に合わせて読む。
- **CI 期待値**: push 前に上記ビルド/テスト 4 コマンドと `cargo run -p neko-assistant -- test-mcp` をローカル実行し、結果を PR 説明に明記する。


