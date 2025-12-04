# コーディング規約 — 小分けモジュール化と開発手法

このドキュメントは本プロジェクト（neko_no_te）で採用する開発手法とコーディング規約の要点をまとめたものです。
特に「小分け（モジュール分割）」をプロジェクト方針として明記し、実装・レビュー・コミットの際に従ってください。

目的

- 可読性の向上
- レビューのしやすさ（差分が小さい）
- 再利用性と単体テストの容易さ
- CI 実行時間の短縮（小さな変更ごとのテスト）

基本方針

- 1 つの変更は「1 つの責務」に限定する。1機能＝1コミットを原則とする。
- ファイルは小さく保つ：1つのソースファイルは原則として単一の概念（例：1つのサブモジュール、1つの主要構造体、1つの責務）を担う。
- Rust のモジュールレイアウトは `src/<module>/mod.rs` と `src/<module>/<sub>.rs` のように分割する。トップレベルが大きくなりそうなら単一ファイルにしない。

モジュール分割のガイドライン

- 機能ごとにフォルダを作る（例：`src/plugins/`、`src/gui/`）。その中でさらに `discovery.rs`, `enabled.rs`, `metadata.rs` のように責務毎に分割する。
- 公開 API は `mod.rs`（もしくは `lib.rs`）でまとめて re-export する。内部の実装はプライベートまたは個別のサブモジュールとして切る。
- テストは可能な限りファイル単位で書く。サブモジュール毎に `#[cfg(test)] mod tests` を用意するか、`tests/` ディレクトリで統合テストを用意する。

命名規則

- ファイル名は snake_case、型は PascalCase。
- モジュール（フォルダ）名は機能名（例：`plugins`、`gui`）。サブモジュールはその責務を短く示す（例：`discovery`, `enabled`, `metadata`）。

コミットとレビュー

- 小さな差分でコミットする（機能追加・バグ修正・リファクタのいずれでも、できるだけ分割）。
- コミットメッセージは `type(scope): short description` 形式を推奨（例：`feat(plugins): add plugin.toml parsing`）。
- PR には必ず変更点の要約とテスト手順・影響範囲を記載する。
- PR 説明には `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace`, `cargo run -p neko-assistant -- test-mcp` の実行結果をチェックリスト形式で必ず含め、未実行の場合は理由を明記する。

リファクタ時の注意

- リファクタであっても振る舞いが変わらないことを示すためにユニットテスト／統合テストを更新・追加する。
- 大きな構造変更は小さいステップに分けて PR を出す（例：1) API 抽出 2) 実装移行 3) 削除）。

GPUI ヘルパーとビュー モデル運用

- 新規 UI ヘルパー（メニュー、ツールバー、ポップアップなど）が `ChatView` の状態を参照する場合、必ず `MenuContext` や `ToolbarViewModel` のような専用ビュー モデルを経由して依存を受け渡す。`ChatView` のフィールドへ直接アクセスするヘルパーはレビューで差し戻す。
- 共有データ（`repo_root`, `plugins`, `ChatController` など）は `MenuContext` が `Arc` を再利用する形で受け取り、ウィジェット側はクローン済みのハンドルのみを触る。重複 clone や `Arc` の多重生成はテストで検出する。
- ビュー モデルを導入したら `#[cfg(test)]` モジュールで軽量ユニットテストを追加し、表示テキストやトグルラベルなどウィジェットが依存する出力が期待通りであることを確認する。`MenuContext::new_for_testing` 等のテスト用 API を活用して GPUI 非依存のまま検証する。
- 既存ウィジェットへ機能追加する場合も同じルールを適用し、ドロップダウンやボタンの listener がビュー モデルの状態にのみ触れるように保つ。

Feature フラグとオプション依存

- CLI のように選択的にビルドする機能は `Cargo.toml` の features 機能で管理する。
- optional dependency を features に紐付け、デフォルトは軽量にする。

CI とドキュメント

- 重要な変更を加えたら `docs/` と `TESTS.md` を更新すること（CI が参照するため）。
- ドキュメントは日本語で記述する。短く明確に。

## Phase 5: MCP統合の進捗（2025-12-02）

### 実装完了項目

- ✅ **mcp_client.rs** (308行): JSON-RPC 2.0通信、tokio非同期プロセス管理
- ✅ **mcp_manager.rs** (127行): 複数MCPサーバー統合管理
- ✅ **設定システム**: 実行ファイルと同じディレクトリの `mcp_servers.json`
- ✅ **テストコマンド**: `cargo run -- test-mcp`
- ✅ Windows環境対応（npx.cmd）

詳細は `docs/development/phase5-mcp-integration.md` を参照。

### 既知の問題

- **receive_responseのタイムアウト**: ツール一覧取得時にフリーズ（BufReader::read_line()が無限待機）
  - 対策: `tokio::time::timeout()`でタイムアウト実装
  - デバッグ: レスポンス内容のeprintln!出力

### 次回実装予定（Phase 5継続）

1. **receive_responseのデバッグとタイムアウト**
   - JSON区切りでの読み込み実装
   - エラーハンドリング改善

2. **LangChain統合**
   - MessageHandlerでMcpManager初期化
   - システムプロンプトにツール説明追加
   - LLM応答からツール呼び出しをパース

3. **ツール実行フロー**
   ```
   User → LLM → Tool Call → McpManager → Result → LLM → Response
   ```

### イベント駆動設計の改善提案

現在のMessageHandlerは手続き的な実装だが、以下の改善を検討：

#### 提案: イベント駆動アーキテクチャ

```rust
// イベント定義
enum ChatEvent {
    UserMessageReceived(String),
    LlmResponseReceived(String),
    ToolCallRequested { server: String, tool: String, args: Value },
    ToolResultReceived { tool: String, result: Value },
    ConversationSaved,
    Error(String),
}

// イベントハンドラー
struct EventBus {
    tx: mpsc::UnboundedSender<ChatEvent>,
    rx: Arc<Mutex<mpsc::UnboundedReceiver<ChatEvent>>>,
}

// 各コンポーネントはイベントを購読
impl MessageHandler {
    fn handle_event(&self, event: ChatEvent) {
        match event {
            ChatEvent::UserMessageReceived(msg) => { /* LLM呼び出し */ }
            ChatEvent::LlmResponseReceived(response) => { /* ツール呼び出し検出 */ }
            ChatEvent::ToolCallRequested { .. } => { /* MCP実行 */ }
            ChatEvent::ToolResultReceived { .. } => { /* LLMに返送 */ }
            _ => {}
        }
    }
}
```

#### メリット

- **責務分離**: 各コンポーネントは特定のイベントにのみ反応
- **テスト容易性**: モックイベントで単体テスト可能
- **拡張性**: 新しいイベント追加が容易（例: VoiceInputReceived）
- **デバッグ性**: イベントログでフロー追跡可能

#### 実装時の注意

- Phase 3の教訓を適用：UI層はイベント発火のみ、処理はバックグラウンド
- `Arc<Mutex<>>` の保持時間を最小化
- イベントハンドラーは10行以内を目標

この設計は次回リファクタ時に検討・実装する。


補足（例）

- 先に行った `neko-assistant` の `src/plugins` 分割はこの方針に従った例です：`metadata.rs`（型定義）、
 `discovery.rs`（発見ロジック）、`enabled.rs`（有効化ロジック）に分割し、`mod.rs` で公開 API を再エクスポートしています。

関連ファイル

- `.github/copilot-instructions.md` — このファイルを読んだら開発規約も必ず確認する旨を追記しています。

運用

- 新しくプロジェクトに参加する開発者はこのドキュメントを最初に読むこと。
- 規約の改善案があれば PR を作成してください（ドキュメント自体も同じプロセスで変更します）。
