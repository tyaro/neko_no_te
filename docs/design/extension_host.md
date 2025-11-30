# ExtensionHost（簡易設計） — neko_no_te 向け骨子

目的

- `zed-fork` の拡張（Extension）設計を参考に、`neko_no_te` に段階的に導入可能な「簡易 ExtensionHost」設計を示す。
- 要求: manifest 読込、capability ベースの権限チェック、外部プロセス実行のガード、WASM 実行の取り込みインタフェース、既存プラグイン方式との互換性を保つこと。

設計方針（高レベル）

- 段階導入: 最初は manifest スキーマ拡張 + capability チェック + 外部プロセス実行ガードを実装。中期で archive/install、WASM 実行を追加。長期で extension host（プロキシ）を導入。
- 最小の安全性: 危険な操作（プロセス起動、ネットワークアクセス、ファイルシステムへの書き込みなど）は `capabilities` に明示し、ホスト側で拒否可能にする。
- 互換性: 既存の `plugin.toml` を基礎とし、拡張フィールドは互換性を保って追加する（後方互換）。

目次

- Manifest（拡張 schema）
- ExtensionHost の責務
- Capability チェック（モデル）
- 外部プロセス実行ガード
- WASM 取り込みインタフェース（概要）
- 実装スケッチ（Rust trait / 型）
- ランタイム配置・検索（既存方式との整合）
- テスト・移行計画
- セキュリティ / ライセンス注意

- 1 Manifest（拡張 schema）
  
- 既存 `plugin.toml` の最小フィールド: `name`, `description`, `version`, `author`。
- 追加推奨フィールド（任意、TOML 形式）:

```toml
name = "my-plugin"
description = "Short description"
version = "0.1.0"
author = "Alice <alice@example.com>"

# 追加フィールド
repository = "https://github.com/owner/repo"
icon = "icon.svg"

[capabilities]
# 真偽値ではなく列挙で許可する操作を宣言
process_exec = true
network = false
filesystem_write = false

[targets]
# 将来の自動インストール用の配布指定（省略可）
windows_x86_64 = { archive = "https://...zip", cmd = "myplugin.exe", sha256 = "..." }
```

- `capabilities` はキー名で列挙し、ホスト側は whitelist/validation を行う。

- 2 ExtensionHost の責務
- Manifest の検証（スキーマ・型・バージョン検査）。
- Capability の検証とランタイム拒否（`process_exec` 等）。
- 拡張のロード（ファイルコピー、展開、必要時インストール）。
- 外部実行（archive 展開または process 起動）の安全なラッパー。
- 将来的に WASM を扱うための抽象インタフェースを提供。

- 3 Capability チェック（モデル）
- capability は manifest で宣言される。許可されていない操作を拡張が要求した場合、ホストはエラーを返す。
- recommended capabilities（最小セット）:
  - `process.exec` — 外部プロセス実行
  - `network` — ネットワークアクセス
  - `filesystem.read` / `filesystem.write` — ファイル読み書き
  - `wasm` — WASM 実行を要求
  - `language_server` — 言語サーバ起動（将来）

- 4 外部プロセス実行ガード
- ホストは `spawn_process(manifest, cmd, args, env)` のラッパーを提供し、manifest の capabilities を検査する。
- もし `process.exec` が manifest に含まれていなければ、実行は拒否しエラーを返す。
- 実行時はログと最小限の環境変数のみを渡す、必要ならサンドボックス（chroot / job object 等）を検討。

- 5 WASM 取り込みインタフェース（概要）
- 構成: manifest に `wasm = true` もしくは `capabilities.wasm = true` を追加。
- Interface:
  - `ExtensionHost::instantiate_wasm(manifest, wasm_bytes) -> Result<WasmInstanceHandle>`
  - `WasmInstanceHandle.call(func_name, args) -> Result<Value>`
- 実行環境: 最初は Wasmtime 等の既存ランタイムを外部依存として用いる（feature-gated）。将来的にサンドボックス化やメモリリミット、API surface の制限を追加。

- 6 実装スケッチ（Rust 型 / trait）

```rust
// manifest の読み取り結果（簡易）
#[derive(Debug, Deserialize)]
pub struct SimpleManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub repository: Option<String>,
    pub capabilities: Option<HashMap<String, bool>>,
}

pub trait ExtensionHost {
    /// Load manifest from path and validate schema
    fn load_manifest(&self, path: &Path) -> anyhow::Result<SimpleManifest>;

    /// Check whether the manifest allows a capability (e.g., process.exec)
    fn manifest_allows(&self, manifest: &SimpleManifest, cap: &str) -> bool;

    /// Spawn a process on behalf of extension (guarded)
    fn spawn_guarded(
        &self,
        manifest: &SimpleManifest,
        cmd: &str,
        args: &[&str],
    ) -> anyhow::Result<()>
    ;

    /// (Optional) instantiate a WASM extension
    fn instantiate_wasm(&self, manifest: &SimpleManifest, wasm_bytes: &[u8]) -> anyhow::Result<()>;
}
```

- 7 ランタイム配置・検索（既存方式との整合）

  - 優先順位:
    1. 実行ファイル横の `plugins/`（既に discovery で優先）
    2. リポジトリの `plugins/`（開発用）
    3. インストール済み拡張の専用ディレクトリ（将来）
  - `enabled.json` は location 毎に参照する（既に実装済みの loading を拡張して exe 隣の enabled.json を優先）。

    1) テスト・移行計画
       - 単体テスト: manifest の読み込み・バリデーション、capability チェック、spawn_guarded の拒否パターン／許可パターン。
       - 結合テスト: ダミー拡張（小さなバイナリ）を `crates/plugins/dummy` に作り、`scripts/sync-plugins.ps1` で target に配置して `cargo run` で起動・操作確認。
       - 移行: 既存 `plugin.toml` を新 schema にマッピングする変換ツール（簡易）を提供する。

    2) セキュリティ / ライセンス注意
       - capability による明示的許可を導入しても、拡張が潜在的に危険な動作を行う可能性があるため、ユーザー確認（UI）とログを必須とする。
       - zed-fork のコードを直接流用する場合は、各クレートのライセンス（GPL 等）を精査すること。商用利用や配布には注意が必要。

    3) 段階的導入ロードマップ（短期→中期→長期）
        - 短期 (1-2 週): manifest スキーマ拡張 + discovery の manifest バリデーション + spawn_guarded の最小実装 + UI 上で capability 表示/警告。
        - 中期 (1-2 月): archive-based install（targets）、基本的な WASM 実行インタフェース、インストール UI/ログ。
        - 長期 (数月): ExtensionHost/Proxy の分離、WASM サンドボックス、署名検証、自動更新、より詳細な capability モデル。

    4) 参考実装案（小さなタスク）
        - `docs/design/plugins.md` に manifest 変更案を追記（PR） — まずは manifest 拡張だけを提案。
        - `neko-assistant` に manifest バリデータを追加（`plugins/validation.rs`）
        - `neko-assistant` に `spawn_guarded` ラッパーを追加し、CLI の有効化/無効化で警告を出す。

まとめ

- zed-fork の設計は強力だが重い。neko ではまず manifest の拡張と capability チェック、
- spawn_guarded の導入で大きな安全性向上と将来の拡張余地を確保するのが現実的かつ効果的です。

---
更新履歴: 初版（簡易 ExtensionHost 骨子）
