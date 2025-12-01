# langchain-rust 調査レポート

**調査日**: 2025年12月1日  
**対象**: [langchain-rust](https://github.com/Abraxas-365/langchain-rust) v4.6.0  
**目的**: neko_no_teアプリケーションへの適合性評価

## 1. プロジェクト概要

### 基本情報
- **リポジトリ**: Abraxas-365/langchain-rust
- **最新バージョン**: v4.6.0 (2024年10月7日)
- **Stars**: 1.2k
- **Contributors**: 31名
- **License**: MIT
- **使用プロジェクト**: 126件

### 説明
LangChainのRust実装。LLMを使ったアプリケーションを構成可能なコンポーネントで構築するフレームワーク。

## 2. 主要機能

### 2.1 LLMサポート
- ✅ **OpenAI** (GPT-4, GPT-3.5等)
- ✅ **Azure OpenAI**
- ✅ **Ollama** ← **neko_no_teで使用中**
- ✅ **Anthropic Claude**
- ✅ **Qwen**
- ✅ **Deepseek**

### 2.2 Embeddingsサポート
- OpenAI Embeddings
- Azure OpenAI Embeddings
- **Ollama Embeddings** (nomic-embed-text等)
- FastEmbed (ローカル実行)
- MistralAI Embeddings

### 2.3 Vector Stores
- OpenSearch
- PostgreSQL (pgvector)
- Qdrant
- SQLite (sqlite-vss, sqlite-vec)
- SurrealDB

### 2.4 Chains（チェーン機能）
- **LLM Chain**: 基本的なLLM呼び出しチェーン
- **Conversational Chain**: 会話履歴を管理
- **Conversational Retriever Chain**: RAG（検索拡張生成）
- **Sequential Chain**: 複数のチェーンを順次実行
- **Q&A Chain**: 質問応答
- **SQL Chain**: SQL生成・実行

### 2.5 Agents（エージェント機能）
- Chat Agent with Tools
- OpenAI Compatible Tools Agent
- 利用可能なツール:
  - SerpAPI/Google検索
  - DuckDuckGo検索
  - Wolfram/Math
  - コマンドライン実行
  - Text2Speech

### 2.6 Document Loaders
- PDF (PdfExtractLoader, LoPdfLoader)
- Pandoc (Word, Markdown等)
- HTML / HTML to Markdown
- CSV
- Git commits
- Source code

### 2.7 Semantic Routing
- Static Routing
- Dynamic Routing

## 3. neko_no_teとの適合性

### 3.1 現在のアーキテクチャ

#### neko_no_te
```rust
crates/
  model-provider/     # HTTP通信・認証
  model-adapter/      # 入出力整形
  ollama-client/      # Ollama専用クライアント
  chat-engine/        # 会話管理
```

#### langchain-rust
```rust
src/
  llm/
    ollama/client.rs  # Ollama LLM実装
    openai/mod.rs     # OpenAI互換API
  embedding/
    ollama/           # Ollama Embeddings
  chain/              # チェーン実装
  agents/             # エージェント実装
```

### 3.2 メリット

#### ✅ 即座に利用可能な機能
1. **会話チェーン**: `ConversationalChain`で履歴管理が組み込み済み
2. **RAG機能**: Vector Store + Retriever Chainでドキュメント検索
3. **エージェント**: ツール呼び出し機能が実装済み
4. **Ollama統合**: 既に完全サポート

#### ✅ 実装例が豊富
- `examples/` ディレクトリに50以上のサンプルコード
- Ollama使用例:
  - `examples/llm_ollama.rs`
  - `examples/embedding_ollama.rs`
  - `examples/conversational_chain.rs`

#### ✅ Pure Rust実装
- 非同期処理対応 (tokio)
- 型安全
- パフォーマンス最適化

### 3.3 デメリット・懸念点

#### ⚠️ 重複するコード
- **ollama-client/**: langchain-rustの`ollama_rs`クレートと重複
- **chat-engine/**: ConversationalChainと機能が重複
- **model-adapter/**: langchain-rustのLLMトレイトと異なるアプローチ

#### ⚠️ アーキテクチャの再設計が必要
現在の3層構造（Provider → Adapter → Engine）を、langchain-rustのパターン（LLM → Chain → Agent）に移行する必要がある。

#### ⚠️ 依存関係の増加
```toml
[dependencies]
langchain-rust = { version = "4.6.0", features = ["ollama"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
```

追加の依存:
- `ollama_rs` (langchain-rustが内部で使用)
- `async-openai` (OpenAI互換API用)
- その他多数

#### ⚠️ プロジェクトの成熟度
- 最終リリース: 2024年10月 (約2ヶ月前)
- 活発な開発中
- **破壊的変更のリスク**: メジャーバージョン未満

#### ⚠️ GPUIとの統合
- langchain-rustは非同期処理中心
- GPUIのUI更新との同期が複雑になる可能性

### 3.4 具体的な使用例（neko_no_te向け）

#### 現在のchat-engine置き換え例

```rust
use langchain_rust::{
    chain::{Chain, ConversationalChainBuilder},
    llm::ollama::Ollama,
    memory::SimpleMemory,
    schemas::Message,
};

// Ollamaクライアント初期化
let ollama = Ollama::default().with_model("phi4-mini:3.8b");

// 会話チェーン作成（履歴管理付き）
let memory = SimpleMemory::new();
let chain = ConversationalChainBuilder::new()
    .llm(ollama)
    .memory(memory.into())
    .build()
    .unwrap();

// メッセージ送信
let result = chain
    .invoke(prompt_args! {
        "input" => "こんにちは",
    })
    .await?;
```

#### RAG機能追加例

```rust
use langchain_rust::{
    embedding::ollama::OllamaEmbedder,
    vectorstore::{sqlite_vss::StoreBuilder, VectorStore},
    chain::ConversationalRetrieverChainBuilder,
};

// Embeddings初期化
let embedder = OllamaEmbedder::default()
    .with_model("nomic-embed-text");

// Vector Store初期化
let store = StoreBuilder::new()
    .embedder(embedder)
    .connection_url("docs.db")
    .build()
    .await?;

// ドキュメント検索チェーン
let chain = ConversationalRetrieverChainBuilder::new()
    .llm(ollama)
    .retriever(store.as_retriever(5))
    .build()?;
```

## 4. 導入シナリオ

### 4.1 段階的導入（推奨）

#### Phase 1: 評価・検証
1. `research/langchain-rust-test/` で機能検証
2. 既存コードと並行して動作確認
3. パフォーマンス測定

#### Phase 2: 部分的置き換え
1. **chat-engine**をConversationalChainに置き換え
2. 既存のollama-clientは維持
3. model-adapterとの互換性を確認

#### Phase 3: RAG機能追加
1. Vector Storeの導入
2. Document Loadersの統合
3. プラグインドキュメント検索

#### Phase 4: エージェント機能
1. ツール呼び出しの実装
2. MCPサーバーとの統合
3. プラグインツールの拡張

### 4.2 全面導入（リスク高）

#### メリット
- 最新のLangChain機能をすべて利用可能
- コード量の大幅削減
- エージェント・RAG機能が即座に使える

#### デメリット
- 既存コードの大規模書き換え
- GPUIとの統合で予期しない問題
- 学習コスト

## 5. 技術的詳細

### 5.1 Ollama統合

#### LLM使用
```rust
use langchain_rust::{
    language_models::llm::LLM,
    llm::ollama::Ollama,
};

let ollama = Ollama::default().with_model("phi4-mini:3.8b");
let response = ollama.invoke("Hello!").await?;
```

#### ストリーミング
```rust
let mut stream = ollama.stream(&messages).await?;
while let Some(data) = stream.next().await {
    match data {
        Ok(stream_data) => {
            print!("{}", stream_data.content);
        }
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
```

### 5.2 依存関係の詳細

#### 必須
```toml
[dependencies]
langchain-rust = "4.6.0"
serde_json = "1.0"
```

#### Ollama機能
```toml
langchain-rust = { version = "4.6.0", features = ["ollama"] }
```

#### Vector Store（オプション）
```toml
langchain-rust = { version = "4.6.0", features = ["sqlite-vss"] }
# または
langchain-rust = { version = "4.6.0", features = ["sqlite-vec"] }
```

### 5.3 パフォーマンス考慮点

1. **非同期処理**: tokioランタイム必須
2. **メモリ使用量**: Vector Storeは大量メモリを消費
3. **レスポンス時間**: Embeddings生成に時間がかかる

## 6. 推奨事項

### ✅ 導入を推奨する場合
- **RAG機能が必要**: ドキュメント検索を実装したい
- **エージェント機能が必要**: 複雑なツール呼び出しを実装したい
- **複数のLLMプロバイダ**: Ollama以外も使いたい
- **チェーン機能**: 複雑な処理フローが必要

### ⚠️ 慎重に検討すべき場合
- **シンプルなチャットアプリ**: 現在のアーキテクチャで十分
- **安定性重視**: 破壊的変更のリスクを避けたい
- **コード理解優先**: 自前実装で学習効果を高めたい

### ❌ 導入を推奨しない場合
- **MVP段階**: 基本機能の実装中
- **リソース不足**: 大規模リファクタリングの時間がない

## 7. 結論と次のステップ

### 7.1 総合評価

| 項目 | 評価 | 備考 |
|------|------|------|
| 機能性 | ⭐⭐⭐⭐⭐ | RAG、Agent、Chain等すべて揃っている |
| Ollama対応 | ⭐⭐⭐⭐⭐ | 完全サポート |
| ドキュメント | ⭐⭐⭐⭐ | 豊富な例、ただし日本語ドキュメントは少ない |
| 成熟度 | ⭐⭐⭐ | 活発だが破壊的変更の可能性 |
| neko_no_te適合性 | ⭐⭐⭐⭐ | 現在のアーキテクチャと競合するが、将来性は高い |

### 7.2 推奨アプローチ

**段階的導入を推奨**:

1. **検証フェーズ（1-2週間）**
   - `research/langchain-rust-test/` で動作確認
   - パフォーマンス測定
   - GPUIとの統合テスト

2. **部分導入（Phase 2）**
   - ConversationalChainで既存のchat-engineを置き換え
   - 既存コードと並行運用
   - 問題なければ本格移行

3. **機能拡張（Phase 3-4）**
   - RAG機能の追加
   - プラグインドキュメント検索
   - エージェント機能の実装

### 7.3 次のアクション

- [ ] `research/langchain-rust-test/` ディレクトリを作成
- [ ] Ollama + ConversationalChainの動作確認
- [ ] 既存chat-engineとのベンチマーク比較
- [ ] GPUIとの統合テスト
- [ ] 導入判断（Go/No-Go決定）

## 参考リンク

- [langchain-rust GitHub](https://github.com/Abraxas-365/langchain-rust)
- [langchain-rust ドキュメント](https://langchain-rust.sellie.tech/get-started/quickstart)
- [Ollama Examples](https://github.com/Abraxas-365/langchain-rust/tree/main/examples)
- [Discord Community](https://discord.gg/JJFcTFbanu)
