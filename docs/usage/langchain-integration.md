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

### 設定データベース

設定はアプリ実行ファイルと同じディレクトリにある `neko_assistant_settings.db`（SQLite）へ保存されます。
旧 `config.toml` が存在する場合は初回起動時に読み込み・移行されます。

内容を確認する場合は、`sqlite3` などで以下を実行してください（Windows の例）。

```powershell
sqlite3.exe .\target\debug\neko_assistant_settings.db "SELECT * FROM app_config;"
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

2. 設定データベースを確認:
   ```powershell
   sqlite3.exe .\target\debug\neko_assistant_settings.db "SELECT use_langchain FROM app_config;"
   ```

3. ログを確認:
   アプリのログに "LangChain mode enabled" が表示されているか確認

### 設定が保存されない

`neko_assistant_settings.db` を配置しているディレクトリ（例: `target\debug`）に書き込み権限があるか確認してください。
