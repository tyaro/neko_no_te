# Phi-4 Mini Adapter Plugin

`phi4-mini-adapter` は Microsoft の [Phi-4-mini-instruct](https://huggingface.co/microsoft/Phi-4-mini-instruct) 用にチューニングされた `ModelAdapter` 実装です。<br/>
`chat-core` / `neko-assistant` からは、プラグインとして読み込むだけで Phi-4 のチャットテンプレートやツール呼び出しフォーマットを自動的に整形できます。

## 提供機能

- `<|system|> ... <|end|>` ベースの Phi-4-mini-instruct プロンプト構築
- `<|tool|> ... <|/tool|>` ブロックでのツール(JSON) 埋め込み
- `phi4-mini:3.8b` / `Phi-4-mini-instruct` のモデル名を両方サポート

## 使い方

1. プロジェクトルートでプラグインをビルド:

  ```powershell
  cargo test -p phi4-mini-adapter
  ```

2. プラグインフォルダを `target/<config>/plugins` にコピー（開発時はスクリプト推奨）:

  ```powershell
  pwsh .\scripts\sync-plugins.ps1 -Configuration Debug
  ```

3. `neko-assistant` を起動すると、`plugins/phi4-mini-adapter/plugin.toml` が検出され、対応モデルにアダプタが紐づきます。

## 実装メモ

- アダプタの責務は「プロンプト整形」と「ツール定義のフォーマット」のみです。ネットワーク処理は `model-provider` に委譲しています。
- 詳細なチャットテンプレート仕様は `docs/design/phi4-integration.md` を参照してください。

## テスト

モック Provider を用いたユニットテストを `src/lib.rs` に同梱しています。`cargo test -p phi4-mini-adapter` で実行できます。
