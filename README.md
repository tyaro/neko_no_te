# neko_no_te

neko_no_te は Rust で書かれた AI アシスタント（GUI）プロジェクトのモノリポジトリです。主な目的は会話管理とプラグインによる機能拡張を提供することです。

## 主要機能

- 🤖 **AI チャット**: Ollama を使用した対話型チャット
- 🔌 **プラグインシステム**: 動的なツールプラグインのロード
- ⚙️ **設定管理**: GUI での柔軟な設定変更
- 🧪 **LangChain 統合**: 実験的な LangChain-rust サポート（設定で切り替え可能）
- 🛰️ **MCP サーバー管理**: GUI から Model Context Protocol サーバーを登録・編集

## 主要コンポーネント

- `neko-assistant/` — メインの GUI クレート。会話の管理・プラグイン読み込みを担います。
- `crates/langchain-bridge/` — LangChain-rust 統合クレート（実験的機能）
- `plugins/` — 動的にロードされるプラグイン群。README に「ビルド無しで追加・削除可能」との記載があるため実行時ロード方式を確認してください。
- `docs/` — プロジェクト全体のドキュメント。各クレートにも `docs/` を置く運用を採用しています。

クイックスタート（開発環境: PowerShell）

```powershell
# ルートでビルド
cargo build --workspace

# 全テスト
cargo test --workspace

# 個別クレートのテスト例
cargo test -p neko-assistant

# フォーマットと静的解析
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

## MCP サーバー管理 (GUI)

`neko-assistant` のチャット画面ツールバーに「Manage MCP」ボタンを追加しました。これにより以下の操作を GUI 上で実行できます。

1. **一覧表示**: 現在登録されている MCP サーバーが名前・起動コマンドとともに表示されます。
2. **編集/削除**: 各行の `Edit` ボタンでフォームへロードし、`Remove` で即削除できます。
3. **追加/保存**: フォーム内で `Server Name / Command / Arguments / Env` を入力し、`Save Entry` を押すと `neko-assistant.exe`（または `cargo run` 時は `target/debug/`）と同じディレクトリにある `mcp_servers.json` に永続化されます。
4. **環境変数**: `Env` フィールドは `KEY=value` を改行区切りで入力する形式です。空欄の場合は `null` として保存されます。

> スクリーンショットは現在準備中です。必要であれば `cargo run -p neko-assistant` を実行して UI を直接確認してください。

開発ルール（要点）

- 機能ごとにクレートを作成することを推奨します。
- 各クレートに `docs/` と `TESTS.md` を置き、変更時はドキュメント更新を行ってください。
- 変更は最小差分で、1機能＝1コミットを心がけてください。

プラグインに関する注意

- `plugins/` 内のプラグインはランタイムに読み込まれる設計です。新しいプラグインを追加する場合は `neko-assistant` のプラグインローダ実装を参照してください（`neko-assistant/src` 内で `plugin` を検索）。

リポジトリ間の扱い

- `zed-fork/` は参照用に置かれています。`zed-fork` のコードは別管理のため、ここで変更・push しないでください。

ライセンス

- 本リポジトリは MIT ライセンスです（`LICENSE` を参照）。

貢献ガイド（簡易）

- 変更前に関連ドキュメントを更新し、プルリクエストで差分を提示してください。
- PR 前に `cargo test --workspace` を必ず実行してください。

追加の質問や項目の修正があれば教えてください。
