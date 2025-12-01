# LangChain 統合ガイド

## 概要

neko-assistant では、設定画面から LangChain の使用を切り替えることができます。

## 機能

- ✅ 設定画面での LangChain 使用切り替え
- ✅ `langchain-bridge` クレートによる統合
- ✅ 既存実装との並行動作

## 使用方法

### 1. 設定画面を開く

GUIのメニューから Settings を開きます。

### 2. LangChain 使用を切り替え

「Use LangChain (experimental)」チェックボックスで切り替えます。

- ☐ オフ（デフォルト）: 既存の ollama-client 実装を使用
- ☑ オン: LangChain-rust による実装を使用

### 3. 設定を保存

「Save Settings」ボタンをクリックして保存します。

### 4. アプリ再起動

設定を反映するためにアプリを再起動してください。

## 実装詳細

### 設定ファイル

設定は `~/.config/neko-assistant/config.toml` に保存されます：

```toml
ollama_base_url = "http://localhost:11434/"
default_model = "phi4-mini:3.8b"
max_history_messages = 100
use_langchain = false  # または true
```

### アーキテクチャ

```
┌─────────────────┐
│  chat.rs        │ ← use_langchain 設定を読み取り
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
┌───▼────┐ ┌─▼──────────────┐
│ chat-  │ │ langchain-     │
│ engine │ │ bridge         │
└────────┘ └─┬──────────────┘
              │
         ┌────▼──────┐
         │ langchain-│
         │ rust      │
         └───────────┘
```

### 今後の実装

現在は設定切り替えのみ実装されています。今後の開発予定：

1. **Phase 2: 実際のLLM呼び出し** (1週間)
   - chat.rs で LangChainEngine を使用
   - 会話履歴の保持
   - エラーハンドリング

2. **Phase 3: ストリーミング対応** (1週間)
   - リアルタイムトークン表示
   - UIへの統合

3. **Phase 4: 完全移行** (1週間)
   - 旧実装の削除
   - パフォーマンス最適化

## 参照

- [LangChain-rust 評価ドキュメント](./research/langchain-rust-evaluation.md)
- [検証プロジェクト](../research/langchain-rust-test/)
- [langchain-bridge クレート](../crates/langchain-bridge/)

## トラブルシューティング

### LangChain モードが動作しない

1. Ollama が起動しているか確認:
   ```powershell
   ollama list
   ```

2. 設定ファイルを確認:
   ```powershell
   cat ~/.config/neko-assistant/config.toml
   ```

3. ログを確認:
   アプリのログに "LangChain mode enabled" が表示されているか確認

### 設定が保存されない

設定ディレクトリの書き込み権限を確認してください：
```powershell
ls ~/.config/neko-assistant/
```
