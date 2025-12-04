# Phi-4 モデル統合ガイド

## 概要

このドキュメントは Microsoft Phi-4 モデルファミリー（`phi4-mini:3.8b`）を neko_no_te で使用する際のプロンプト形式とベストプラクティスをまとめたものです。

参照: 
- [microsoft/phi-4 - Hugging Face](https://huggingface.co/microsoft/phi-4)
- [microsoft/Phi-4-mini-instruct - Hugging Face](https://huggingface.co/microsoft/Phi-4-mini-instruct)

## 外部アダプタプラグイン

- 実装場所: `crates/plugins/phi4-mini-adapter`
- 役割: `<|system|> / <|user|> / <|assistant|>` テンプレートと `<|tool|> ... <|/tool|>` ブロックを自動組み立てし、`phi4-mini:3.8b` と `Phi-4-mini-instruct` の両モデル名をサポート。
- `plugin.toml` メタデータで kind=adapter, models=[...] を宣言。`scripts/sync-plugins.ps1` を実行すると `target/<config>/plugins/phi4-mini-adapter` に同期され、実行バイナリが自動検出する。
- 追加の詳細・テスト手順は `crates/plugins/phi4-mini-adapter/README.md` を参照。

## モデル仕様（Phi-4-mini-instruct）

| 項目 | 内容 |
|------|------|
| モデル名 | Phi-4-mini-instruct (phi4-mini:3.8b) |
| パラメータ数 | 3.8B |
| コンテキスト長 | 128K tokens（Phi-4 の 16K から大幅拡張） |
| 語彙サイズ | 200,064 tokens（多言語サポート強化） |
| アーキテクチャ | Dense decoder-only Transformer、Grouped-Query Attention |
| 主な用途 | 推論、ロジック、数学、コード生成、会話、**関数呼び出し** |
| 対応言語 | 22言語（英語、日本語、中国語、韓国語、スペイン語、フランス語など） |
| ライセンス | MIT |
| リリース日 | 2025年2月 |

## プロンプト形式（チャットテンプレート）

Phi-4-mini-instruct は **簡潔な**チャット形式を使用します（Phi-4 とは異なる形式）。以下のトークンで会話を構造化します：

### 基本構造（Phi-4-mini-instruct）

```
<|system|>
{system_message}<|end|>
<|user|>
{user_message}<|end|>
<|assistant|>
{assistant_response}<|end|>
```

### トークンの意味

- `<|system|>` — システムメッセージの開始
- `<|user|>` — ユーザーメッセージの開始
- `<|assistant|>` — アシスタント応答の開始
- `<|end|>` — 各ターンの終了

**注意**: Phi-4 と Phi-4-mini-instruct は**異なるトークンフォーマット**を使用します。
- **Phi-4**: `<|im_start|>role<|im_sep|>content<|im_end|>`
- **Phi-4-mini-instruct**: `<|role|>content<|end|>`

### 実装例

#### シンプルな会話（Phi-4-mini-instruct 形式）

```
<|system|>
You are a helpful assistant.<|end|>
<|user|>
What is Rust?<|end|>
<|assistant|>
```

#### システムプロンプト付きの例

```
<|system|>
You are an expert Rust programmer.<|end|>
<|user|>
Write a function to reverse a string.<|end|>
<|assistant|>
```

#### マルチターン会話

```
<|system|>
You are a helpful coding assistant.<|end|>
<|user|>
Write a Rust function to calculate factorial.<|end|>
<|assistant|>
Here's a recursive factorial function in Rust:

fn factorial(n: u64) -> u64 {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}<|end|>
<|user|>
Can you make it iterative?<|end|>
<|assistant|>
```

## ツール呼び出し（Function Calling）形式

Phi-4-mini-instruct は**ネイティブな関数呼び出し**をサポートしています。これは LangChain や MCP 統合に非常に重要です。

### ツール定義の形式

システムプロンプト内で `<|tool|>` と `<|/tool|>` タグでツールを定義します：

```
<|system|>
You are a helpful assistant with some tools.
<|tool|>
[
  {
    "name": "get_weather_updates",
    "description": "Fetches weather updates for a given city using the Weather API.",
    "parameters": {
      "city": {
        "description": "The name of the city for which to retrieve weather information.",
        "type": "str",
        "default": "London"
      }
    }
  }
]
<|/tool|>
<|end|>
<|user|>
What is the weather like in Paris today?<|end|>
<|assistant|>
```

### ツール定義の構造

各ツールは JSON オブジェクトで定義：

```json
{
  "name": "function_name",
  "description": "What the function does",
  "parameters": {
    "param_name": {
      "description": "Parameter description",
      "type": "str|int|float|bool|list|dict",
      "default": "optional_default_value"
    }
  }
}
```

### 実装例：複数ツール

```
<|system|>
You are a helpful assistant with access to these tools.
<|tool|>
[
  {
    "name": "get_current_weather",
    "description": "Get the current weather in a given location",
    "parameters": {
      "location": {
        "description": "The city and state, e.g. San Francisco, CA",
        "type": "str"
      },
      "unit": {
        "description": "Temperature unit",
        "type": "str",
        "default": "celsius"
      }
    }
  },
  {
    "name": "search_web",
    "description": "Search the web for information",
    "parameters": {
      "query": {
        "description": "The search query",
        "type": "str"
      },
      "num_results": {
        "description": "Number of results to return",
        "type": "int",
        "default": 5
      }
    }
  }
]
<|/tool|>
<|end|>
<|user|>
What's the weather in Tokyo and show me news about AI?<|end|>
<|assistant|>
```

### モデルの応答形式

モデルは関数呼び出しを JSON 形式で返します：

```json
{
  "name": "get_current_weather",
  "parameters": {
    "location": "Tokyo, Japan",
    "unit": "celsius"
  }
}
```

複数の関数呼び出しも可能：

```json
[
  {
    "name": "get_current_weather",
    "parameters": {
      "location": "Tokyo, Japan"
    }
  },
  {
    "name": "search_web",
    "parameters": {
      "query": "AI news"
    }
  }
]
```

## Rust での実装

### Phi4MiniAdapter の実装方針

`crates/model-adapter/` に `Phi4MiniAdapter` を作成し、プロンプトとツール呼び出しを適切にフォーマットします。

```rust
pub struct Phi4MiniAdapter;

impl Phi4MiniAdapter {
    /// 会話履歴を Phi-4-mini-instruct 形式にフォーマット
    fn format_chat_prompt(
        system: Option<&str>,
        messages: &[(Role, &str)],
    ) -> String {
        let mut prompt = String::new();
        
        // システムメッセージ
        if let Some(sys) = system {
            prompt.push_str("<|system|>\n");
            prompt.push_str(sys);
            prompt.push_str("<|end|>\n");
        }
        
        // 会話履歴
        for (role, content) in messages {
            let role_tag = match role {
                Role::User => "<|user|>",
                Role::Assistant => "<|assistant|>",
                Role::System => "<|system|>",
            };
            prompt.push_str(&format!("{}\n", role_tag));
            prompt.push_str(content);
            prompt.push_str("<|end|>\n");
        }
        
        // アシスタントの応答を開始
        prompt.push_str("<|assistant|>\n");
        
        prompt
    }
    
    /// ツール定義を Phi-4-mini-instruct 形式にフォーマット
    fn format_tools(tools: &[ToolSpec]) -> String {
        let tools_json: Vec<serde_json::Value> = tools
            .iter()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name,
                    "description": tool.description.as_ref().unwrap_or(&String::new()),
                    "parameters": tool.schema.as_ref().unwrap_or(&serde_json::json!({}))
                })
            })
            .collect();
        
        format!("<|tool|>\n{}\n<|/tool|>", serde_json::to_string_pretty(&tools_json).unwrap())
    }
}

#[async_trait]
impl ModelAdapter for Phi4MiniAdapter {
    fn adapter_name(&self) -> &str {
        "phi4-mini-adapter"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "phi4-mini:3.8b".to_string(),
            "Phi-4-mini-instruct".to_string(),
        ]
    }

    async fn invoke(
        &self,
        provider: &dyn ModelProvider,
        model: &str,
        prompt: &str,
        tools: Option<&[ToolSpec]>,
    ) -> Result<GenerateResult, ProviderError> {
        let mut system_prompt = String::from("You are a helpful assistant.");
        
        // ツールがある場合はシステムプロンプトに追加
        if let Some(t) = tools {
            if !t.is_empty() {
                system_prompt.push_str(" with access to these tools.\n");
                system_prompt.push_str(&Self::format_tools(t));
            }
        }
        
        // プロンプトをフォーマット
        let formatted = Self::format_chat_prompt(
            Some(&system_prompt),
            &[(Role::User, prompt)],
        );
        
        let result = provider.generate(model, &formatted).await?;
        
        // 関数呼び出しのパース（結果が JSON 形式の場合）
        if let Some(structured) = &result.structured {
            if structured.get("name").is_some() {
                // 単一の関数呼び出し
                return Ok(result);
            } else if structured.is_array() {
                // 複数の関数呼び出し
                return Ok(result);
            }
        }
        
        Ok(result)
    }
}
```

### ToolSpec の定義

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: Option<String>,
    pub schema: Option<serde_json::Value>,
}

impl ToolSpec {
    /// 簡易的なツール定義
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: Some(description.to_string()),
            schema: None,
        }
    }
    
    /// パラメータ付きツール定義
    pub fn with_parameters(name: &str, description: &str, parameters: serde_json::Value) -> Self {
        Self {
            name: name.to_string(),
            description: Some(description.to_string()),
            schema: Some(parameters),
        }
    }
}

// 使用例
let weather_tool = ToolSpec::with_parameters(
    "get_weather",
    "Get current weather for a location",
    serde_json::json!({
        "location": {
            "description": "City name",
            "type": "str"
        },
        "unit": {
            "description": "Temperature unit",
            "type": "str",
            "default": "celsius"
        }
    })
);
```

### ChatEngine での使用

```rust
// アダプタとプロバイダの初期化
let adapter = Box::new(Phi4MiniAdapter);
let provider = Box::new(OllamaProvider::new("http://localhost:11434/")?);

// ツール定義
let tools = vec![
    ToolSpec::with_parameters(
        "get_weather",
        "Get current weather for a location",
        serde_json::json!({
            "location": {
                "description": "City name",
                "type": "str"
            }
        })
    ),
];

// チャットエンジンの作成
let mut engine = ChatEngine::new(provider, adapter);

// 通常の会話
let response = engine.send_message("Hello!").await?;

// ツール付き会話
let response_with_tools = engine.send_message_with_tools(
    "What's the weather in Tokyo?",
    Some(&tools)
).await?;

// 関数呼び出しの検出
if let Some(function_call) = response_with_tools.structured {
    let function_name = function_call["name"].as_str().unwrap();
    let parameters = &function_call["parameters"];
    
    // ツールを実行
    let tool_result = execute_tool(function_name, parameters).await?;
    
    // 結果をモデルに返す
    let final_response = engine.send_tool_result(tool_result).await?;
}
```

## ベストプラクティス

### 1. システムプロンプトの設定

適切なシステムプロンプトでモデルの振る舞いを制御します：

```rust
// 一般的なアシスタント
"You are a helpful assistant."

// コーディング支援
"You are an expert Rust programmer. Provide clear, idiomatic code examples."

// 専門知識
"You are a senior software architect specializing in distributed systems."
```

### 2. 会話履歴の管理

- **コンテキスト長**: 16K tokens を意識して履歴を管理
- **トリミング**: 古いメッセージから削除してコンテキストを収める
- **要約**: 長い会話は要約して保持

### 3. エラーハンドリング

```rust
match engine.send_message(user_input).await {
    Ok(response) => { /* 成功処理 */ }
    Err(ChatError::ContextTooLong) => {
        // 履歴をトリミングして再試行
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

### 4. 温度パラメータ

- **0.0-0.3**: 正確さ重視（コード生成、数学）
- **0.5-0.7**: バランス（一般的な会話）
- **0.8-1.0**: 創造性重視（物語生成）

## Ollama での使用

### モデルの取得

```bash
# Ollama でモデルをダウンロード
ollama pull phi4-mini:3.8b

# 動作確認
ollama run phi4-mini:3.8b "Hello, how are you?"
```

### API エンドポイント

```
POST http://localhost:11434/api/generate
Content-Type: application/json

{
  "model": "phi4-mini:3.8b",
  "prompt": "<|im_start|>user<|im_sep|>\nHello<|im_end|>\n<|im_start|>assistant<|im_sep|>\n"
}
```

## 制限事項と注意点

### 言語サポート

- **主言語**: 英語、多言語サポート（22言語）
- **日本語**: サポートあり（Phi-4 より改善）
- **多言語データ**: トレーニングデータに含まれる

### コード生成

- **得意**: Python、一般的なパッケージ（`typing`, `math`, `collections` 等）
- **注意**: 他の言語やレアなパッケージは手動検証が必要

### 推論とロジック

- **強み**: 数学的推論、論理的思考、関数呼び出し
- **ベンチマーク**: 
  - MATH (64.0)
  - GSM8K (88.6)
  - HumanEval（コード生成）
  - Function Calling（Berkeley benchmark）

### 関数呼び出しの注意点

- **幻覚**: モデルが存在しない関数名や URL を生成する可能性
- **検証**: 関数呼び出しの結果は必ず検証する
- **エラーハンドリング**: 不正な JSON や存在しない関数への対処が必要

### 安全性

- **フィルタリング**: 有害コンテンツのフィルタリングあり
- **バイアス**: 多言語データだが英語中心のためバイアスの可能性
- **検証**: 高リスクシナリオでは追加の検証が必要
- **長い会話**: 非常に長い会話では繰り返しや一貫性の問題が発生する可能性

## 参考資料

- [Phi-4 Model Card](https://huggingface.co/microsoft/phi-4)
- [Phi-4-mini-instruct Model Card](https://huggingface.co/microsoft/Phi-4-mini-instruct)
- [Phi-4 Technical Report (arXiv)](https://arxiv.org/pdf/2412.08905)
- [Phi-4-mini Technical Report (arXiv)](https://arxiv.org/pdf/2503.01743)
- [Ollama Documentation](https://github.com/ollama/ollama)
- [Phi Cookbook (GitHub)](https://github.com/microsoft/PhiCookBook)

## 更新履歴

- 2024-12-01: 初版作成（Phi-4 プロンプト形式とベストプラクティス）
- 2024-12-01: Phi-4-mini-instruct のツール呼び出し形式を追加
