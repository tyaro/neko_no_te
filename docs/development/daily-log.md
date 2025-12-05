# Daily log — 進捗まとめ

日付: 2025-12-06

このドキュメントは、このワークスペースで行った最近の作業をまとめた短いデイリーログです。主にプラグイン検出／プロンプトビルダー・テストランナー／CLI 機能に関連する変更を行いました。

---

## 概要（要点）
- プラグイン検出と ModelSelector の改善
- phi4-mini 用の PromptBuilder プラグイン（`crates/plugins/phi4-mini-prompt-builder`）を作成
- 動的プラグインの ABI 脆弱性を回避するための JSON-FFI フック (`build_prompt_json`) を実装
- `neko-assistant` に CLI テストラッパーを追加（thin wrapper）／本体のテストランナー実装は `research/cli-test-runner` に移設
- `research/cli-test-runner` にスクリプトベースのプロンプトビルダー検証ランナーを実装（正規表現、agent mode, prompt_variables, expected_tool_names 等をサポート）
- `crates/app-config` に sqlite バックエンドでのトークン保存（set/get/list/remove）を実装し、`neko-assistant` CLI に `token` サブコマンドを追加
- DB ファイルは .gitignore に追加（`**/neko_assistant_settings.db`）

---

## 主要変更ひとことメモ（時系列）

1. プラグイン探索の仕組みを改善
   - shared library の存在による自動発見を実装（enabled.jsonに頼らない）
   - Console UI に plugin→model のマッチログを出力するようにした

2. phi4-mini prompt builder（プラグイン）を追加
   - `crates/plugins/phi4-mini-prompt-builder` を追加
   - PromptBuilder トレイト実装、factory、加えて安全な JSON-FFI (`build_prompt_json`) を実装
   - ユニットテストを追加し、ローカルでパスすることを確認

3. CLI 版のテストランナーを実装 & 安全化
   - `neko-assistant` に一時的に CLI 実行用 `cli_test.rs` を作っていたが、後に research に移設
   - `research/cli-test-runner` を新規に追加（ワークスペースに登録）
   - ランナーはまずプラグインの JSON-FFI を試し、失敗すればホストの builtin prompt builders を利用（クロス dylib の trait オブジェクト呼び出しでの UB を回避）

4. 検証・テスト機能拡張
   - テストスクリプト形式 `scripts/cli-tests/phi4_prompt_builder_tests.json` を追加
   - runner に `expected_prompt_contains`, `expected_prompt_regex`（flags 付き）、`expected_agent_mode`、`expected_prompt_variables`、`expected_tool_names` を実装
   - CLI からの呼び出し方法（thin wrapper）を `neko-assistant` に提供

5. アプリ内トークン保存（env 不使用）
   - `crates/app-config` の sqlite DB に `tokens` テーブルを追加
   - set/get/delete/list の API を実装、ユニットテストを追加
   - `neko-assistant` CLI に `token` サブコマンドを追加（add/get/list/remove）
   - DB のファイル名を `.gitignore` に追加

---

## 実行サンプル / 検証コマンド

- research runner の直接実行:
```pwsh
cargo run -p cli-test-runner -- scripts/cli-tests/phi4_prompt_builder_tests.json --use-target-plugins
```
- neko-assistant 経由（thin wrapper）:
```pwsh
cargo run -p neko-assistant -- --cli cli-test --script scripts/cli-tests/phi4_prompt_builder_tests.json --use-target-plugins
```
- token CLI の例:
```pwsh
# 追加
cargo run -p neko-assistant -- --cli token add openai default sk-EXAMPLE
# 取得
cargo run -p neko-assistant -- --cli token get openai default
# 一覧
cargo run -p neko-assistant -- --cli token list
# 削除
cargo run -p neko-assistant -- --cli token remove openai default
```

---

## 現状の状態（完了 / 保留）
- phi4-mini プラグイン作成: 完了
- research CLI テストランナー: 実装とサンプル検証完了
- neko-assistant: thin wrapper 実装済み、不要な直接テストモジュールは削除済み
- app-config token store: 実装済み、ユニットテスト通過
- DB は .gitignore へ追加済み

残作業（将来的に検討）:
- トークンの平文保存を改善（OS keyring / 暗号化） — 推奨
- runner の出力を CI向け JSON に整形
- プラグインの ABI 関連ドキュメント整備（JSON-FFI の使い方と注意点）

---

## 注意 / メモ
- Linux/Windows 間の shared library、trait オブジェクトの呼び出しは不安定になり得るため、JSON-FFI を用いた安全な経路を優先しています。
- 現在トークンはアプリ内 sqlite に平文で保存されています。セキュリティ要件に応じて keyring・暗号化の導入をおすすめします。

---

もしよければ、次にどれを優先するか教えてください（例: トークン暗号化 or keyring 統合 / runner を CI 対象にする / runner の JSON 出力追加）。
