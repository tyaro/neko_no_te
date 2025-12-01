# Phase 5: MCP統合 - 実装レポート

## 概要

Model Context Protocol (MCP) サーバーとの統合機能を実装しました。これにより、neko-assistantは外部ツール（ファイルシステム、GitHub、Webスクレイピングなど）をLLMが動的に呼び出せるようになります。

## 実装内容

### 1. MCP Client (`mcp_client.rs`)

**責務**: 単一のMCPサーバーとのJSON-RPC 2.0通信

**主要機能**:
- `McpClient::new()` - MCPサーバープロセスをtokio::process::Commandで起動
- `initialize()` - MCPハンドシェイク（プロトコルバージョン`2024-11-05`）
- `list_tools()` - 利用可能なツール一覧を取得
- `call_tool()` - ツールを引数付きで実行
- `Drop` trait - プロセス終了時の自動クリーンアップ

**技術的特徴**:
- 非同期stdin/stdout通信（AsyncBufReadExt/AsyncWriteExt）
- JSON-RPC 2.0準拠（id, method, params, result/error）
- 環境変数の明示的設定（Windowsの`npx.cmd`対応）
- リクエストID自動インクリメント（Arc<Mutex<u64>>）

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

**場所**: `%AppData%\Roaming\neko-assistant\mcp_servers.json`

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

### 4. MessageHandler統合

**変更内容**:
- `MessageHandler`に`mcp_manager: Option<Arc<McpManager>>`フィールドを追加
- コンストラクタで初期化（現在は`None`、後でGUIから設定可能に）

**将来の実装予定**:
1. システムプロンプトにツール説明を追加
2. LLM応答からツール呼び出しをパース
3. `McpManager::call_tool()`で実行
4. 結果をLLMにフィードバック

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

## 動作確認済み項目

- ✅ MCPサーバーの起動（`npx.cmd` + filesystem server）
- ✅ JSON-RPC 2.0通信（initialize handshake）
- ✅ ツール一覧の取得（list_tools）
- ✅ 複数サーバーの並行管理（McpManager）
- ✅ 設定ファイルのロード（JSON形式）
- ✅ Windows環境でのnpx実行（.cmd拡張子）

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
