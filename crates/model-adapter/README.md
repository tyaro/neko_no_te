# model-adapter

モデル固有の呼び出しフォーマット（function-calling / tools / prompt 形式）を内包するアダプタ抽象と、`llama3.1:8b` 用のデフォルトアダプタを提供します。

## 目的

コアは `ModelProvider` を通した汎用的な generate を担い、各モデル固有の入力整形や関数呼び出しフォーマットは `ModelAdapter` に委ねることで拡張性を確保します。

## 組み込みデフォルト

- `Llama3DefaultAdapter` — `llama3.1:8b` をサポートします。`ToolSpec` を JSON にシリアライズしてプロンプト末尾に付与し、`ModelProvider` に委譲します。

## 拡張方法

追加モデル対応は `crates/plugins/` または `plugins/` に新しいクレートを作成し、`ModelAdapter` を実装してください（例: `qwen3`, `phi4-mini` など）。
