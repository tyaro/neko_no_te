# 変更ログ - 2024

## [未リリース]

### 追加
- `crates/neko-ui`: カスタムUIコンポーネントライブラリ
  - `TextInput`: IME対応の複数行テキスト入力コンポーネント
  - `ChatBubble`: チャットメッセージ表示用バブルコンポーネント
- `crates/ui-utils`: IMEとスクロール機能の共通クレート
  - `TextInputState`: テキスト入力状態管理（IME対応）
  - `TextInputHandler`: トレイト定義
  - `impl_entity_input_handler!`: EntityInputHandler自動実装マクロ
  - `ScrollManager`: スクロール管理ヘルパー

### 修正
- 入力欄のフォーカス問題を修正
  - `track_focus` を入力エリア自体に設定
  - クリックでフォーカス移動が正しく動作
  - キーボード入力が受け付けられるように

### 変更
- `neko-assistant/src/gui/chat.rs`: UIコンポーネントを使用するようリファクタリング
  - `MessageType` を `neko-ui` から使用
  - `ChatBubble` コンポーネントでチャットバブルを描画
  - 約200行のコード削減（33%減）

### 改善
- 入力欄の初期フォーカス実装（起動時に入力欄にフォーカス）
- 入力欄のクリック対応（クリックでフォーカス移動）
- IBeam カーソルスタイルで入力エリアを明示
- コンポーネントの再利用性向上

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
