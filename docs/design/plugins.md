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

- 「adapter」種別のプラグインは `model-adapter::ModelAdapter` トレイトを実装すること。
- `supported_models()` で対応するモデル名（例: `"qwen3:8b"`）を返すこと。

> **Prompt Builder との棲み分け**
>
> function-calling の書式を差し替える等、プロンプト構築自体を拡張したい場合は新設予定の Prompt Builder SPI（`docs/design/prompt-builder-spi.md`）を利用する。Adapter プラグインは Provider とのやり取り（モデル固有の HTTP パラメータ変換など）に特化させる。

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

## Manifest（plugin.toml）拡張案

既存の `plugin.toml` は最小限のメタ情報しか持ちません。安全性・配布・UI 表示性を高めるため、以下の拡張フィールドを提案します。後方互換性を保つため、すべて任意フィールドとします。

例:

```toml
name = "my-adapter"
description = "Adapter for ExampleModel"
version = "0.1.0"
author = "Alice <alice@example.com>"
repository = "https://github.com/owner/my-adapter"
icon = "icon.svg" # 相対パス（オプション）

[capabilities]
process_exec = false    # 外部プロセス実行を要求するか
network = true          # ネットワークアクセスを要求するか
filesystem_write = false
wasm = false            # WASM 実行を必要とするか

[targets.windows_x86_64]
archive = "https://example.com/my-adapter-windows-x86_64.zip"
cmd = "my-adapter.exe"
sha256 = "..."
```

推奨ルール:

- `capabilities` はプラグインが行う可能性のある敏感操作を列挙する（真偽値）。ホストは manifest を参照して実行を拒否可能にする。
- `targets` は将来の自動インストールのための配布情報。省略可能。
- UI は `repository` と `icon` を使って詳細表示を行う。

次のステップとして、`neko-assistant` の discovery に manifest バリデーション（スキーマチェック）を追加し、capabilities を表示・警告する UI を実装することを推奨します。
