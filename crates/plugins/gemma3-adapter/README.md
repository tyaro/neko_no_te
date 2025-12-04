# adapter-template

このテンプレートは、`neko_no_te` 用の `ModelAdapter` を実装する方法を示します。

クイックスタート

1. このディレクトリを `crates/plugins/<your-adapter>` にコピーします。
2. `Cargo.toml` のメタデータ（`name`、`authors`、`description` など）を更新します。
3. `src/lib.rs` にモデル固有のシリアライズ／リクエスト形成処理を実装します（TODO の箇所を参照）。
4. 動作確認のために `cargo test -p <your-adapter>` を実行します。

プラグインメタデータ

- クレートルートに `plugin.toml` を置き、以下の必須フィールドを含めてください。
  - `name`: プラグイン／クレート名
  - `description`: UI に表示する短い説明（1 行）
  - `version`: SemVer 文字列（例: "0.1.0"）
  - `author`: 作成者名と連絡先（例: "Alice <alice@example.com>"）

crates.io への公開

- 公開前に `Cargo.toml` を適切に更新してください（ユニークな `name`、説明、`authors`、`license` など）。
- テストが通ることを確認してください: `cargo test -p <your-adapter>`。
- 公開手順の一例:

```powershell
# 初回ログイン（未ログインの場合）:
cargo login <your-api-token>
# crate フォルダもしくはワークスペースルートから公開:
cd crates/plugins/<your-adapter>
cargo publish --allow-dirty
```

注意: 開発中は `--allow-dirty` を一時的に使うことがありますが、正式なリリース時には外してください。また、`package.metadata` や `README.md` が crates.io 表示に適切であることを確認してください。

補足

- 実装の参考としてワークスペース内の `model-adapter` と `model-provider` を参照してください。
- Adapter の責務は主にリクエストの整形とレスポンスのパースに集中させ、ネットワークやトランスポートは `provider` 側で扱うようにしてください。
