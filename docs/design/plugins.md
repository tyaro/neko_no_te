<!--
Plugin authoring guide for model adapters
Location: docs/design/plugins.md
-->

# プラグイン（Adapter）作成ガイド

目的
- 外部開発者が本プロジェクト向けの ModelAdapter プラグインを容易に作成・テスト・公開できるよう、実装テンプレートと手順を提供します。

ディレクトリと公開方法
- 当面はワークスペース内で `crates/plugins/<adapter-name>` として作成してください。
- 将来的に動的ロードをサポートする場合は `plugins/` 配下にビルド済みライブラリを置く方針に変更できます。

必須事項
- プラグインは `model-adapter::ModelAdapter` トレイトを実装すること。
- `supported_models()` で対応するモデル名（例: `"qwen3:8b"`）を返すこと。

簡単な作成手順
1. `crates/plugins/adapter-template` をコピーして新しいディレクトリ名に変更する。
2. `Cargo.toml` の `name` と `description` を更新する。
3. `src/lib.rs` 内の実装箇所（`TODO` コメント）を編集してモデル固有のシリアライズや function-calling の整形を実装する。
4. `cargo build -p <crate-name>` でビルド、`cargo test -p <crate-name>` でテストを実行する。

パブリッシング（外部共有）
- リポジトリ外部で配布したい場合は、GitHub レポジトリとして公開し、README に互換バージョンや依存バージョンを明記してください。

テストの方針
- Adapter は Provider をモックできるべきです。`ModelProvider` を実装するダミーを用意し、`invoke` の振る舞いを検証してください。
- CI では adapter のユニットテストを実行し、モデル固有の統合テストは環境依存のため別ワークフローで実行します。

互換性とバージョニング
- `ModelAdapter` の API に互換性を保つため、重大な変更はメジャーバージョン上げを伴います。
- プラグイン README に対応する `model-adapter` の最小バージョンを明記してください。

例: 新しい adapter を追加する流れ
1. adapter を `crates/plugins/<name>` に追加（`ModelAdapter` 実装）。
2. `cargo test -p <name>` を実行。
3. PR を作成し、レビュー → merge。
4. リリース手順がある場合はタグ付け・リリース。

付録: テンプレートの場所
- `crates/plugins/adapter-template` が公式テンプレートです。ここからコピーして開始してください。
