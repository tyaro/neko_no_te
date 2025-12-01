# Ollama チャット GUI アプリ 実装計画（更新版）

## 現状分析（2025年12月1日更新）

### ✅ 既に完成している部分
- **GUI フレームワーク**: GPUI + gpui-component で基本的な UI 構造あり
- **プロバイダ層**: `model-provider` + `ollama-client` で Ollama 通信可能
- **アダプタ層**: `model-adapter` で `llama3.1:8b` デフォルトサポート
- **チャット UI**: `neko-assistant/src/gui/chat.rs` に完成版実装
  - チャットバブル（左右寄せ、最大幅600px）
  - 複数行入力（Enter送信、Shift+Enter改行）
  - スクロール対応
- **プラグイン管理**: `plugins/` モジュールで発見・有効化ロジック実装済み
- **LangChain統合**: `langchain-bridge` クレートで langchain-rust v4.6.0 を使用
  - ✅ Ollama との実際の通信完了
  - ✅ `send_message_simple()` で基本的な会話可能
  - ⚠️ UIブロッキング実行（非同期化は今後の課題）
- **設定管理**: `app-config` クレートで Ollama URL、モデル名管理
  - ✅ 設定画面（Settings）実装済み
  - ✅ LangChain ON/OFF 切り替え可能
  - ✅ TOML ファイル保存（`~/.config/neko-assistant/config.toml`）

### 🔄 改善が必要な部分
- **非同期処理**: LLM呼び出しが同期実行でUIがフリーズする
- **会話履歴**: `ConversationalChain` 未使用（毎回独立した会話）
- **エラーハンドリング**: 基本的な実装のみ（タイムアウト、リトライなし）
- **ストリーミング**: 未実装

### ❌ まだ未実装の部分
- **会話履歴の永続化**: メモリ内のみ、保存・読み込み機能なし
- **MCP統合**: Model Context Protocol サーバー連携
- **RAG機能**: 検索拡張生成
- **マークダウンレンダリング**: プレーンテキスト表示のみ

## 更新された実装計画

### フェーズ3: 会話履歴と非同期処理改善 🎯
**目標**: 会話の永続化と UX 改善

#### 3.1 会話履歴機能
**優先度**: 高

**実装内容**:
```rust
// crates/chat-history/src/lib.rs
#[derive(Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<Message>,
}

pub struct ConversationManager {
    storage_path: PathBuf,  // ~/.neko-assistant/conversations/
}

impl ConversationManager {
    pub fn save(&self, conv: &Conversation) -> Result<()>;
    pub fn load(&self, id: &str) -> Result<Conversation>;
    pub fn list(&self) -> Result<Vec<ConversationMetadata>>;
    pub fn delete(&self, id: &str) -> Result<()>;
}
```

**GUI変更**:
- サイドバーに会話一覧を追加
- 新規会話ボタン
- 会話切り替え機能

#### 3.2 非同期処理の改善
**優先度**: 中

現在の問題点：
```rust
// 現在: UIをブロック
let result = std::thread::spawn(|| {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { /* ... */ })
}).join().unwrap();
```

**改善策**:
GPUIの非同期パターンを調査し、適切な実装に移行：
- `cx.background_executor().spawn()` の活用
- チャンネルでUI更新を通知
- ローディングインジケーター表示

#### 3.3 会話記憶の有効化
**優先度**: 高

現在は `send_message_simple()` を使用しており、毎回独立した会話。
`langchain-bridge` の `send_message()` に切り替え：

```rust
// langchain-bridge/src/lib.rs
pub async fn send_message(&mut self, user_input: &str) -> Result<String> {
    let chain = ConversationalChainBuilder::new()
        .llm(self.ollama.clone())
        .memory(self.memory.clone())
        .build()
        .expect("Failed to build chain");
    
    let response = chain.invoke(user_input).await?;
    Ok(response)
}
```

**GUI変更**:
- 会話履歴をUI上部に表示（折りたたみ可能）
- 会話のクリアボタン

### フェーズ4: ストリーミングとマークダウン ✨
**目標**: リアルタイム応答と見やすい表示

#### 4.1 ストリーミング対応
**優先度**: 中

`ollama-client` にストリーミングAPI追加：
```rust
pub async fn generate_stream(
    &self,
    model: &str,
    prompt: &str,
) -> impl Stream<Item = Result<String, Error>> {
    // POST /api/generate with stream=true
}
```

`langchain-rust` のストリーミングサポートを調査・活用。

#### 4.2 マークダウンレンダリング
**優先度**: 低

- `pulldown-cmark` でマークダウンをHTMLに変換
- GPUIでのカスタムレンダラー実装
- コードブロックのシンタックスハイライト（`syntect`）

### フェーズ5: MCP統合とツール機能 🚀
**目標**: Model Context Protocol でツール呼び出し

#### 5.1 MCP クライアント実装
**新規クレート**: `crates/mcp-client/`

```rust
pub struct McpClient {
    server_url: String,
    transport: Transport,  // stdio or HTTP
}

pub trait McpTool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value>;
}
```

#### 5.2 LangChain ツール統合
`langchain-rust` の `Tool` トレイトを使い、MCP機能をツールとして公開：

```rust
struct McpToolWrapper {
    mcp_tool: Box<dyn McpTool>,
}

impl Tool for McpToolWrapper {
    async fn call(&self, input: &str) -> Result<String> {
        let params = serde_json::from_str(input)?;
        let result = self.mcp_tool.execute(params).await?;
        Ok(serde_json::to_string(&result)?)
    }
}
```

#### 5.3 ツールUI
- 利用可能なツール一覧表示
- ツール実行履歴
- 実行結果のビジュアライゼーション

### フェーズ6: RAG機能 📚
**目標**: ドキュメント検索拡張生成

#### 6.1 ベクトルストア
**新規クレート**: `crates/vector-store/`

- ローカルベクトルDB（`qdrant` または `chromadb`）
- 埋め込みモデル（Ollama の `nomic-embed-text`）
- ドキュメント分割・インデックス化

#### 6.2 RAG パイプライン
```rust
pub struct RagEngine {
    vector_store: Box<dyn VectorStore>,
    llm: Ollama,
}

impl RagEngine {
    pub async fn query(&self, question: &str) -> Result<String> {
        // 1. クエリ埋め込み
        // 2. 類似ドキュメント検索
        // 3. コンテキスト構築
        // 4. LLM呼び出し
    }
}
```

#### 6.3 ドキュメント管理UI
- ドキュメントアップロード
- インデックス状況表示
- 検索結果プレビュー

## 更新されたディレクトリ構成

```
neko_no_te/
├── neko-assistant/          # メインGUIアプリ
│   └── src/
│       ├── main.rs
│       ├── gui/
│       │   ├── chat.rs      # チャット画面 ✅
│       │   ├── settings.rs  # 設定画面 ✅
│       │   ├── history.rs   # 履歴サイドバー（TODO）
│       │   └── tools.rs     # ツール管理（TODO）
│       └── plugins/         # プラグイン管理 ✅
├── crates/
│   ├── app-config/          # 設定管理 ✅
│   ├── langchain-bridge/    # LangChain統合 ✅
│   ├── chat-history/        # 会話履歴管理（TODO）
│   ├── model-provider/      # プロバイダ抽象 ✅
│   ├── model-adapter/       # アダプタ抽象 ✅
│   ├── ollama-client/       # Ollama HTTP クライアント ✅
│   ├── mcp-client/          # MCP クライアント（TODO）
│   ├── vector-store/        # RAG用ベクトルストア（TODO）
│   └── neko-ui/             # カスタムUIコンポーネント ✅
└── research/                # 機能検証
    ├── rag-prototype/       # RAG検証（TODO）
    └── mcp-integration/     # MCP検証（TODO）
```

## 技術的負債と改善点

### 1. langchain-bridge の警告修正
```rust
// 現在: 未使用フィールド
pub struct LangChainEngine {
    ollama: Ollama,
    base_url: String,  // ⚠️ 警告
    model: String,     // ⚠️ 警告
}
```

**修正**: フィールドを削除し、必要な情報は `Ollama` インスタンスから取得。

### 2. 非同期パターンの統一
現在は `std::thread::spawn` + `tokio::runtime::Runtime::new()` を使用。
GPUIの推奨パターンに統一する必要あり。

### 3. エラーハンドリングの強化
```rust
#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("LangChain error: {0}")]
    LangChain(#[from] anyhow::Error),
    #[error("Connection timeout after {0}s")]
    Timeout(u64),
    #[error("Model not available: {0}")]
    ModelNotFound(String),
    #[error("Rate limit exceeded")]
    RateLimit,
}
```

## 実装優先順位

### 🔥 最優先（Phase 3）
1. **会話履歴の永続化** - ユーザビリティに直結
2. **会話記憶の有効化** - `send_message()` への切り替え
3. **非同期処理の改善** - UIフリーズ解消

### 🌟 次点（Phase 4）
4. ストリーミング対応
5. マークダウンレンダリング

### 🚀 将来（Phase 5-6）
6. MCP統合
7. RAG機能

## 次のステップ

**即座に開始できるタスク**:
1. `crates/chat-history/` クレート作成
2. `ConversationManager` 実装（JSON保存）
3. `langchain-bridge` の `send_message()` を GUI で使用
4. 会話履歴サイドバーのUI実装

**確認が必要な事項**:
- ✅ Ollama は稼働中（`http://localhost:11434/`）
- ✅ 使用モデル: `phi4-mini:3.8b`
- ✅ LangChain統合完了（基本機能）
- 🔄 非同期処理の改善方針（GPUIのベストプラクティス調査）

---

**更新履歴**:
- 2025-12-01: LangChain統合完了を反映、フェーズ1-2をスキップしてフェーズ3から開始
