# CLI Mode Usage Examples

このドキュメントは `neko-assistant` の CLI モードの使い方と、外部スクリプトからの LLM 応答性能検証方法をまとめます。

## 基本的な使い方

### 1. シンプルなチャット実行

```bash
cargo run -p neko-assistant -- chat --prompt "Rustの特徴を教えて"
```

### 2. MCPツール無効化（LLMのみ）

```bash
cargo run -p neko-assistant -- chat --prompt "こんにちは" --no-mcp
```

### 3. JSON出力（スクリプト連携用）

```bash
cargo run -p neko-assistant -- chat \
  --prompt "東京の天気を教えて" \
  --format json
```

出力例:
```json
{
  "response": "...",
  "model": "phi4-mini:3.8b",
  "elapsed_ms": 16654,
  "tool_calls": 1,
  "plugins_enabled": [],
  "mcp_enabled": true
}
```

### 4. 詳細ログ付き実行（デバッグ用）

```bash
cargo run -p neko-assistant -- chat \
  --prompt "大阪の天気" \
  --verbose
```

### 5. LLM生対話出力（デバッグモード）

`--debug` フラグを使うと、LLMに送信される実際のプロンプトと応答を stderr に出力します。

```bash
cargo run -p neko-assistant -- chat \
  --prompt "こんにちは" \
  --debug
```

出力例:
```
=== LangChain Debug Output ===
[DEBUG] User Input:
こんにちは

[DEBUG] Prompt sent to LLM:
あなたは日本語で回答するAIアシスタントです。ツール呼び出し結果や引用した数値があれば、それらを尊重しつつ自然な日本語で簡潔にまとめてください。

ユーザー入力:
こんにちは

[DEBUG] Executing agent...

[DEBUG] Final LLM Response:
はい、私は日本語で応答できるAIアシスタントです...
=== End Debug Output ===
```

MCPツール呼び出し時は、`DEBUG: MCP raw frame:` で生のJSON-RPC通信も確認できます。

## モデル指定

```bash
cargo run -p neko-assistant -- chat \
  --model "llama3.2:3b" \
  --prompt "説明して"
```

## プラグインとMCPの制御

```bash
# MCP無効、プラグイン無効
cargo run -p neko-assistant -- chat \
  --prompt "テスト" \
  --no-mcp \
  --no-plugins

# MCP有効（デフォルト）、プラグイン無効
cargo run -p neko-assistant -- chat \
  --prompt "天気を教えて" \
  --no-plugins
```

## 検証スクリプト

PowerShell スクリプトで複数プロンプトを一括検証:

```powershell
# MCPツールなしで検証
.\scripts\verify-llm.ps1

# MCPツールありで検証
.\scripts\verify-llm.ps1 -EnableMcp

# 別モデルで検証
.\scripts\verify-llm.ps1 -Model "llama3.2:3b" -EnableMcp
```

### カスタム検証スクリプト例

```powershell
# response_test.ps1
$prompts = @(
    "Rustとは？",
    "東京の天気",
    "1+1は？"
)

foreach ($prompt in $prompts) {
    $result = cargo run -p neko-assistant --quiet -- `
        chat --prompt $prompt --format json 2>&1 | Out-String
    
    $json = ($result -match '\{.*\}') | ConvertFrom-Json
    Write-Host "$prompt -> $($json.elapsed_ms)ms"
}
```

## 統合テスト用途

CI/CD パイプラインでの利用例:

```yaml
# .github/workflows/llm-test.yml
- name: Test LLM response
  run: |
    cargo run -p neko-assistant -- chat \
      --prompt "hello" \
      --no-mcp \
      --format json > response.json
    
    # Parse and validate
    $response = Get-Content response.json | ConvertFrom-Json
    if ($response.elapsed_ms -gt 30000) {
      throw "Response too slow"
    }
```

## トラブルシューティング

### MCPツールが見つからない

`mcp_servers.json` が実行ファイルと同じディレクトリに存在するか確認:

```bash
# Debug用に実行ディレクトリを明示
ls target/debug/mcp_servers.json

# 存在しない場合は cargo run の位置から見て配置
cp mcp_servers.json target/debug/
```

### プラグインが読み込まれない

プラグイン同期スクリプトを実行:

```powershell
.\scripts\sync-plugins.ps1 -Configuration Debug
```

### JSON出力がパースできない

cargo の警告が混入している場合は `--quiet` を追加:

```bash
cargo run -p neko-assistant --quiet -- chat \
  --prompt "test" \
  --format json
```

## 関連ドキュメント

- `docs/development/phase5-mcp-integration.md` - MCP統合の詳細
- `docs/design/plugins.md` - プラグインアダプタの仕組み
- `scripts/verify-llm.ps1` - 検証スクリプト本体
