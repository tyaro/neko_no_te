# 変更ログ - 2024

## [未リリース]

### 追加
- **CLIチャットモード**: 汎用的な `chat` サブコマンドを追加し、プロンプト・モデル・MCP/プラグイン有効化を実行時引数で制御可能に
  - `--prompt`, `--model`, `--no-mcp`, `--no-plugins`, `--format (text|json)`, `--verbose` オプションをサポート
  - `--debug` フラグでLLMとの生の対話（プロンプト・レスポンス・ツール呼び出し）を stderr に出力
  - JSON出力モードで外部スクリプトからのLLM応答性能検証が容易に
- **検証スクリプト**: `scripts/verify-llm.ps1` により複数プロンプトでの自動検証とサマリー出力が可能
- **CLI使用ガイド**: `docs/usage/cli-mode.md` にCLI実行例と統合テスト用途を追加
- `crates/neko-ui`: カスタムUIコンポーネントライブラリ
  - `TextInput`: IME対応の複数行テキスト入力コンポーネント
  - `ChatBubble`: チャットメッセージ表示用バブルコンポーネント
- `crates/ui-utils`: IMEとスクロール機能の共通クレート
  - `TextInputState`: テキスト入力状態管理（IME対応）
  - `TextInputHandler`: トレイト定義
  - `impl_entity_input_handler!`: EntityInputHandler自動実装マクロ
  - `ScrollManager`: スクロール管理ヘルパー

### 修正
- MCP初期化成功メッセージを stderr に出力し、JSON出力モードで混入しないよう修正
- MCP ツール名の不一致により LangChain agent がツールを見つけられなかった問題を修正
- 入力欄のフォーカス問題を修正
  - `track_focus` を入力エリア自体に設定
  - クリックでフォーカス移動が正しく動作
  - キーボード入力が受け付けられるように

### 変更
- **プラグイン配置**: アダプタプラグインをソースコードではなくコンパイル済みライブラリ（`.dll`/`.so`/`.dylib`）として配置するよう変更
  - `Cargo.toml` に `crate-type = ["cdylib", "rlib"]` を追加
  - `sync-plugins.ps1` がライブラリファイルと `plugin.toml` のみをコピー
  - 実行時は `target/<config>/plugins/` からバイナリを検出
- LangChain Tool 名から `@server` 接頭辞を削除し、LLM が正しくツール呼び出しできるように変更
- `neko-assistant/src/gui/chat.rs`: UIコンポーネントを使用するようリファクタリング
  - `MessageType` を `neko-ui` から使用
  - `ChatBubble` コンポーネントでチャットバブルを描画
  - 約200行のコード削減（33%減）
- `chat-core::ChatController`: `tokio::sync::watch` で `ChatState` を配信し、UI が push 型で同期できるようにした（2025-12-04）。
- `neko-assistant/src/gui/chat/`: UI スナップショットマッパーを分離し、`neko-ui` コンポーネントへの委譲を強化（2025-12-04）。

### 改善
- 入力欄の初期フォーカス実装（起動時に入力欄にフォーカス）
- 入力欄のクリック対応（クリックでフォーカス移動）
- IBeam カーソルスタイルで入力エリアを明示
- コンポーネントの再利用性向上
- ドキュメント更新: `docs/development/phase5-mcp-integration.md` と `docs/development/coding-guidelines.md` に最新の MCP / PR 手順を追記（2025-12-04）。

## Phase 1 MVP (完了)

### 追加
- `crates/app-config`: 設定管理クレート
- `crates/chat-engine`: チャットエンジンクレート
- `crates/model-adapter`: Phi4MiniAdapter 実装
- `neko-assistant/src/gui/chat.rs`: メインチャット画面

### 機能
- Ollama 経由での AI チャット
- セッション保存/読み込み
- Phi-4 プロンプトフォーマット対応
- Ctrl+Enter / Enter 送信切り替え
- チャットバブル表示（ユーザー/AI/システム/エラー）
- 自動スクロール

## 初期セットアップ

### 追加
- プロジェクト基本構造
- `model-provider`, `ollama-client` クレート
- プラグインディスカバリーシステム
- Copilot Instructions (`.github/copilot-instructions.md`)
