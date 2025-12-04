# Phase 5: MCP統合 - 実装レポート

### 優先度: 高

1. **receive_responseのデバッグ**
   - レスポンスをeprintln!で出力
   - タイムアウト実装（`tokio::time::timeout(Duration::from_secs(10), ...)`）
   - 行区切りではなくJSON区切りで読み込み

2. ✅ **LangChainとの統合（2025-12-02）**
   - MCP ツール→LangChain Tool ブリッジ、`LangChainToolAgent` を導入済み
   - MessageHandler が MCP 有効時に Tool Agent を優先使用

3. ✅ **ツール呼び出しフロー**
   ```
   User: "Read file src/main.rs"
   → LangChain Agent が MCP Tool を function-call として選択
   → McpManager::call_tool("filesystem", "read_file", args)
   → Result JSON をフォーマットしてチャットへ返信
   ```
**コード例**:
```rust
let mut client = McpClient::new("npx.cmd", &["@modelcontextprotocol/server-filesystem".to_string()], None).await?;
client.initialize().await?;
let tools = client.list_tools().await?;
```

### 2. MCP Manager (`mcp_manager.rs`)

**責務**: 複数のMCPサーバーを統合管理

**主要機能**:
- `initialize_all()` - すべてのMCPサーバーを並行起動
- `get_all_tools()` - 全サーバーからツールを集約
- `call_tool(server, tool, args)` - 指定サーバーのツールを実行
- `find_server_for_tool()` - ツール名から適切なサーバーを検索
- `get_tools_description()` - LangChain用のツール説明文を生成

**データ構造**:
- `Arc<Mutex<HashMap<String, McpClient>>>` - スレッドセーフなクライアント管理
- `Vec<McpServerConfig>` - サーバー設定（name, command, args, env）

**コード例**:
```rust
let manager = Arc::new(McpManager::new(configs));
manager.initialize_all().await?;
let all_tools = manager.get_all_tools().await?;
```

### 3. 設定ファイル (`mcp_servers.json`)

**場所**: 実行バイナリと同じディレクトリ（例: `target/debug/mcp_servers.json`）

**形式**:
```json
[
  {
    "name": "filesystem",
    "command": "npx.cmd",
    "args": ["-y", "@modelcontextprotocol/server-filesystem", "D:\\develop"],
    "env": null
  }
]
```

**ロード関数**: `load_mcp_config()` - dirsクレートでクロスプラットフォーム対応

### 4. LangChain Tool 統合（`langchain_tools/` + `langchain-bridge`）

**新規モジュール**:
- `neko-assistant/src/langchain_tools/mcp.rs` — MCP ツールを LangChain の `Tool` トレイトへブリッジする `McpLangChainTool` を実装。`build_mcp_tools()` で `McpManager::get_all_tools()` の結果を `Arc<dyn Tool>` に変換。
- `langchain-bridge/src/lib.rs` — `LangChainToolAgent` を追加。`ConversationalAgentBuilder` + `AgentExecutor` を内包し、`invoke()` で Tool 呼び出し付き会話を実行。

**MessageHandler 更新**:
- `langchain_agent: Arc<tokio::sync::Mutex<Option<LangChainToolAgent>>>` を保持し、MCP 設定が存在する場合は起動時に非同期プリウォーム。
- LangChain モードでは `ensure_tool_agent()` で MCP ツールを初期化し、利用可能なら Tool Agent を用いて応答生成。初期化失敗や MCP 未設定時は従来の `LangChainEngine::send_message_simple()` にフォールバック。
- “Thinking…” メッセージ、結果永続化、UI 更新など既存フローは共通化。

### 5. テストコマンド

**使用方法**:
```powershell
cargo run -p neko-assistant -- test-mcp
```

**出力例**:
```
Loading MCP configuration...
Found 1 MCP server(s)
Initializing MCP servers...
MCP server 'filesystem' initialized successfully
✓ All servers initialized

Fetching available tools...
✓ Found 5 tool(s)

Available tools:
1. [filesystem] read_file - Read the complete contents of a file
2. [filesystem] write_file - Create a new file or overwrite
...
```

### 2025-12-02 時点の実装まとめ

- `neko-assistant/src/mcp_client.rs` に `ensure_mcp_config_path()` と `save_mcp_config()` を追加し、実行ファイルと同じディレクトリ（`cargo run` 時は `target/debug/`）に `mcp_servers.json` を常に生成・保存できるようにした。これにより GUI から設定を更新しても即座に永続化される。
- GPUI 製の MCP 管理画面 `neko-assistant/src/gui/mcp_manager.rs` を新設。既存エントリの一覧表示、編集フォーム（name/command/args/env）、保存・削除、フォーム初期化をサポート。チャット画面ツールバーの「Manage MCP」ボタン（`neko-assistant/src/gui/chat/mod.rs`）からいつでも開ける。
- プラグイン実行ガード `neko-assistant/src/plugins/guard.rs` を拡張し、`exec_with_output()` が `process_exec` capability を検証したうえで標準出力／標準エラーを収集して返すようにした。天気プラグインのようにコマンド出力をUIへ還元するシナリオでも再利用できる。
- 2025-12-04 追記: `chat-core::ChatController` が `tokio::sync::watch` チャンネル経由で `ChatState` をブロードキャストするようになり、UI は `ChatEvent` を受け取ったタイミングで最新スナップショットを即座に取得できる。これに伴い `neko-assistant/src/gui/chat/` では UI 用の `ChatUiSnapshot` を追加し、モデルや MCP 情報の整形を `neko-ui` コンポーネントへ渡す薄いラッパーに整理した。
- 2025-12-05 追記: `neko-assistant/src/gui/chat/initialization.rs` の `ChatViewBuilder` にフォールバック処理を実装。会話ストレージは `ConversationManager::default_storage_dir()` 取得 → 失敗時は `%TEMP%/neko-assistant/conversations` へ退避し、それでも初期化できない場合は `fallback_conversations` 配下で再試行する。MCP 設定は `load_mcp_config()` の失敗を握りつぶして LangChain モードを維持しつつ MCP 管理無しで起動し、`eprintln!` で理由を通知する。`cargo test -p neko-assistant mcp_context_disabled_without_langchain` / `mcp_context_falls_back_on_loader_error` で両フォールバックをカバー済み。

## 動作確認済み項目

- ✅ MCPサーバーの起動（`npx.cmd` + filesystem server）
- ✅ JSON-RPC 2.0通信（initialize handshake）
- ✅ ツール一覧の取得（list_tools）
- ✅ 複数サーバーの並行管理（McpManager）
- ✅ 設定ファイルのロード（JSON形式）
- ✅ Windows環境でのnpx実行（.cmd拡張子）
- ✅ 模擬天気MCPサーバー (`research/mcp-weather-server`)

## クレート棚卸しログ（Phase 5）

- `target/metadata.json`: `cargo metadata --format-version 1` のスナップショット（2025-12-05 取得）。
- `target/cargo-tree.txt`: `cargo tree --workspace` の通常依存グラフ。
- `target/cargo-tree-no-default.txt`: `cargo tree --workspace --no-default-features` の出力。
- `target/cargo-udeps.txt`: `cargo +nightly udeps --workspace` の結果。`mcp-weather-server` の `thiserror`、`neko-ui` の `ui-utils` が未使用候補として検出されたため、今後のクレート棚卸しタスクで要確認。

### 模擬天気MCPサーバーの概要

- `research/mcp-weather-server` に Rust 製の JSON-RPC サーバーを追加。`https://weather.tsukumijima.net/api/forecast/city/{city_code}` を呼び出し、`get_weather_forecast` ツールとして公開する。
- Tool 入力: `city`（日本語／ローマ字）または `city_code`（6桁 JMA コード）。主要都市（東京・大阪・京都・横浜・札幌・名古屋・福岡・那覇）を内蔵テーブルで解決。
- 応答: LangChain 用の `content` テキスト（人が読める要約）と、UI がそのまま表示できる詳細 JSON を返す。
- キャッシュ: 同一都市コードのレスポンスを 5 分間メモリ保持し、API 呼び出しを抑制。
- HTTP: `reqwest` + `rustls` を利用し、User-Agent を `neko-weather-mcp/0.1` で送信。

#### 使い方

1. `cargo build -p mcp-weather-server` でバイナリを生成（`target/debug/mcp-weather-server.exe`）。
2. `target/debug/mcp_servers.json`（または配布バイナリと同じフォルダ）に以下のようなエントリを追加。
    ```json
    {
       "name": "weather",
       "command": "D:/develop/neko_no_te/target/debug/mcp-weather-server.exe",
       "args": [],
       "env": null
    }
    ```
3. `cargo run -p neko-assistant -- test-mcp` を実行して `weather` サーバーが初期化され、`get_weather_forecast` が listed されることを確認。
4. GUI で LangChain モードを有効にすると、会話から「東京の天気は？」のような質問で MCP ツールが選択され、返却 JSON がチャットに表示される。

## 既知の問題と対策

### 1. `receive_response()`のタイムアウト

**問題**: ツール一覧取得時にフリーズ（`Fetching available tools...`で停止）

**原因**: `BufReader::read_line()`が無限待機

**対策案**:
- `tokio::time::timeout()`でタイムアウトを設定
- ストリーム終端検出ロジックを追加
- デバッグ出力でレスポンス内容を確認

### 2. Windowsパス対応

**問題**: `npx`コマンドが見つからない

**解決策**: `.cmd`拡張子を明示（`npx.cmd`）

## 次のステップ（Phase 5継続）

### 優先度: 高

1. **receive_responseのデバッグ**
   - レスポンスをeprintln!で出力
   - タイムアウト実装（`tokio::time::timeout(Duration::from_secs(10), ...)`）
   - 行区切りではなくJSON区切りで読み込み

2. **LangChainとの統合**
   - `message_handler.rs`でMcpManager初期化
   - システムプロンプトにツール説明を追加
   ```rust
   let tools_desc = mcp_manager.get_tools_description().await;
   let system_prompt = format!("You are an AI assistant. Available tools:\n{}", tools_desc);
   ```

3. **ツール呼び出しフロー**
   ```
   User: "Read file src/main.rs"
   → LLM: {"tool": "read_file", "args": {"path": "src/main.rs"}}
   → McpManager::call_tool("filesystem", "read_file", args)
   → Result: "file contents..."
   → LLM: "The file contains..."
   ```

### 優先度: 中

4. **UI feedback**
   - チャット画面に「ツール実行中: read_file...」と表示
   - エラー時のわかりやすいメッセージ

5. **設定UI**
   - MCPサーバーの追加/削除/有効化/無効化
   - 設定ファイルの編集インターフェース

### 優先度: 低

6. **複数サーバー対応のテスト**
   - GitHub MCPサーバー追加
   - ツール名の衝突処理

## 技術的教訓

### 1. tokioのプロセス管理

```rust
// ❌ std::process::Command を使うとAsync不可
let child = std::process::Command::new("npx").spawn()?;

// ✅ tokio::process::Command で非同期対応
let child = tokio::process::Command::new("npx.cmd")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;
```

### 2. 環境変数の明示的設定

```rust
// Windowsでは親プロセスのPATHを継承する必要がある
if let Ok(path) = std::env::var("PATH") {
    command.env("PATH", path);
}
```

### 3. JSON-RPC 2.0の行区切り

MCPプロトコルは1行1リクエスト/レスポンス形式：
```
{"jsonrpc":"2.0","id":1,"method":"initialize",...}\n
{"jsonrpc":"2.0","id":1,"result":{...}}\n
```

## 参考資料

- [Model Context Protocol Specification](https://modelcontextprotocol.io/docs/spec)
- [MCP Filesystem Server](https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem)
- [tokio::process documentation](https://docs.rs/tokio/latest/tokio/process/)

---

**作成日**: 2025-12-02  
**Phase**: 5 (MCP Integration)  
**ステータス**: 基礎実装完了、デバッグ中
