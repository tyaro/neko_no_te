# プロンプトビルダー SPI 設計

## 背景

現状のプラグイン拡張は `ModelAdapter` 実装に限定されており、LLM に渡すプロンプトの組み立てや function-calling の後処理はホスト（`chat-engine` / `neko-assistant` 側）に固定実装されている。Phi4-mini のようにチャットテンプレートやツール定義の書式が特殊なモデルでは、ホスト側を書き換えなければ動作しない。

LangChain 連携時も `LangChainToolAgent` が `format!("{instruction}\nユーザー入力\n...")` でプロンプトを構築しており、モデル固有ロジックを注入する拡張ポイントが存在しない。この制約を解消するため、ホストが会話文脈とツール情報を渡し、外部バイナリがプロンプト構築・レスポンスパースを担う SPI (Service Provider Interface) を新設する。

## ゴール

- モデル固有のプロンプト構築／レスポンス整形コードを外部 DLL (plugins/) へ切り出せるようにする。
- 既存の Phase5 フロー（`MessageHandler` → `ChatEvent`）を壊さずに差し替え可能とする。
- MCP ツール一覧や会話履歴など、ホストが持つ文脈をプラグインへ安全に受け渡す共通フォーマットを定義する。
- プラグインの種類（tool adapter / prompt builder）を manifest で区別し、UI 側に表示できるようにする。

## 非ゴール

- LangChain-rust 自体の挙動変更。
- モデル推論実行 (Provider) のプラグイン化。
- UI から独自に tokio::spawn を行うような制御変更。

## アーキテクチャ概要

```
+---------------------+      SPI呼び出し      +-------------------------+
| neko-assistant      | <--------------------> | Prompt Builder Plugin   |
|  - MessageHandler   | 提供: PromptContext   |  - implements PromptSPI |
|  - PluginRegistry   | 要求: PromptPayload   |                         |
+---------------------+      応答: ParsedFlow  +-------------------------+
```

### 1. Prompt SPI crate

新規 crate `crates/prompt-spi/` を追加し、ホストとプラグインの共有インターフェースを定義する。

```rust
pub struct PromptContext<'a> {
    pub model: &'a str,
    pub locale: &'a str,
    pub conversation: &'a [ConversationTurn<'a>],
    pub tools: &'a [ToolSpec],
    pub system_directives: &'a [SystemDirective],
}

pub trait PromptBuilder {
    fn metadata(&self) -> PromptMetadata;
    fn build(&self, ctx: PromptContext) -> PromptPayload;
    fn parse(&self, raw_output: &str) -> PromptResult;
}
```

- `ConversationTurn` は Role + content の軽量 struct。ロック時間短縮のため `ConversationSnapshot` からコピーする。
- `ToolSpec` は MCP から取得した JSON Schema をそのまま参照（既存 `McpTool` を再利用）。
- `PromptPayload` には LLM へ送る文字列、LangChain 用の `prompt_args!` パラメータ、必要に応じた補助情報（温度推奨値など）を保持する。
- `PromptResult` は `final_answer` と `tool_requests`（`Vec<ToolInvocation>`）の 2 系列を持ち、Phase5 の `ChatEvent` へ変換可能な構造に揃える。

### 2. plugin.toml 拡張

既存 manifest に `kind` フィールドを追加し、`adapter` (既存) / `prompt_builder` (新規) を区別する。`prompt_builder` では以下を必須とする。

```toml
kind = "prompt_builder"
entrypoint = "create_prompt_builder"
models = ["phi4-mini:3.8b"]
```

- `entrypoint` は `extern "C" fn() -> *mut dyn PromptBuilderFactory` をエクスポートするシンボル名。
- `models` は優先適用対象。ホストは `model_name` でマッチングし、複数 Plugin がある場合はユーザー設定か manifest の `priority` で解決する。

### 3. PluginRegistry 拡張

`neko-assistant/src/plugins/` 配下に `prompt_builder.rs` を追加し、ロード済みプラグインを `PromptBuilderRegistry` に登録する。`MessageHandler` は `model_name` をキーに `PromptSessionAdapter` を引く。

```rust
struct PromptSessionAdapter {
    builder: Arc<dyn PromptBuilder>,
    preferred_agent: PromptAgentMode, // e.g. LangChain / direct LLM
}
```

`PromptAgentMode` により、
- LangChain 経由で実行し、`PromptPayload.chain_args` を `AgentExecutor` へ渡すモード。
- Provider へ直接 `PromptPayload.raw_prompt` を送るモード。
を切り替える。fallback は従来ロジック。

### 4. ランタイムフロー

1. `MessageHandler` がユーザーメッセージを受信。
2. 対象モデルに `PromptSessionAdapter` が登録されていれば、`PromptContext` を組み立てて `build()` を呼ぶ。
3. `PromptPayload` を使って LangChain か Provider を呼び出し、出力文字列を取得。
4. `PromptBuilder::parse()` で `PromptResult` を得て Phase5 `ChatEvent` に変換（`ToolCallRequested` など）。
5. MCP ツール呼び出し結果を再び `PromptBuilder::parse()` に渡す hooks を将来的に追加する余地を残す。

### 5. Phi4-mini 用プラグイン例

- `PromptBuilder::build()` で `<|system|>` 形式のテンプレートを生成し、ツール定義は `<|tool|>` ブロックへ埋め込む。
- `PromptBuilder::parse()` で JSON 本体を `serde_json::Value` として受け付け、`tool_requests` に複数アクションを返す実装にする。
- `preferred_agent` は `PromptAgentMode::DirectProvider` を返し、LangChain 側の書式制限をバイパスする。

## セキュリティと互換性

- Prompt Builder は会話履歴全体へアクセスできるため、manifest の `capabilities.prompt_builder = true` を表示し、ユーザー確認の上で有効化する。
- SPI は `ABI = "C"`, `repr(C)` の構造体のみを公開し、Rust バージョン非依存化する。
- フォールバックとしてホストのデフォルトプロンプトを常に保持し、プラグインがロードできない/`parse` エラーを返した場合は安全に切り替える。

## 実装タスク概要

1. `crates/prompt-spi` 追加（trait / struct 定義、ABI ラッパー、ドキュメント）。
2. `plugin.toml` バリデーション更新（`kind`, `entrypoint`, `models`, `capabilities.prompt_builder`）。
3. `neko-assistant/src/plugins/` に Prompt Builder discovery & registry 実装。
4. `MessageHandler` と LangChain 経路の改修：`PromptSessionAdapter` を参照し、`PromptPayload` を適用。
5. テスト: mock プラグイン DLL + integration test (`cargo run -p neko-assistant -- test-prompt-spi` など)。
6. ドキュメント更新（本ファイル + `docs/design/plugins.md` + `docs/development/phase5-mcp-integration.md` の補足）。

## 今後の拡張余地

- `parse_tool_result()` API を追加し、ツールの observation をモデルに返すロジックをカスタム可能にする。
- Prompt Builder を WASM でも提供できるよう、`prompt-spi` に `wasmtime` バックエンドを用意する。
- UI でモデル選択時に「対応している Prompt Builder」を明示し、ユーザーが有効/無効を切り替えられるようにする。
