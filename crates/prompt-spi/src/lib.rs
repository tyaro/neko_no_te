//! Prompt Builder SPI shared between neko-assistant host and external plugins.
//! This crate intentionally keeps the API surface minimal and serialization-friendly.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// コンテキスト内のメッセージロール
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationRole {
    System,
    User,
    Assistant,
    Tool,
}

/// 1 ターンの会話内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn<'a> {
    pub role: ConversationRole,
    pub content: &'a str,
}

/// システムディレクティブの情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDirective<'a> {
    pub source: DirectiveSource,
    pub content: &'a str,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DirectiveSource {
    Host,
    User,
    Plugin,
}

/// MCP ツールやその他ツールの仕様
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

/// PromptBuilder に渡される全体コンテキスト
#[derive(Debug)]
pub struct PromptContext<'a> {
    pub model: &'a str,
    pub locale: &'a str,
    pub conversation: &'a [ConversationTurn<'a>],
    pub tools: &'a [ToolSpec],
    pub system_directives: &'a [SystemDirective<'a>],
}

/// PromptBuilder が返す推論ヒント
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptExecutionHints {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PromptAgentMode {
    LangChain,
    DirectProvider,
}

/// LLM に渡すペイロード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptPayload {
    pub agent_mode: PromptAgentMode,
    pub prompt: Option<String>,
    pub prompt_variables: Map<String, Value>,
    pub execution_hints: PromptExecutionHints,
}

impl PromptPayload {
    pub fn with_prompt(prompt: impl Into<String>, mode: PromptAgentMode) -> Self {
        Self {
            agent_mode: mode,
            prompt: Some(prompt.into()),
            prompt_variables: Map::new(),
            execution_hints: PromptExecutionHints::default(),
        }
    }
}

/// Tool 呼び出し要求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub name: String,
    pub arguments: Value,
}

/// PromptBuilder の解析結果
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptParseOutput {
    pub final_answer: Option<String>,
    pub tool_requests: Vec<ToolInvocation>,
}

/// メタデータ（UI表⽰や衝突解決用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub supported_models: Vec<String>,
    pub homepage: Option<String>,
    pub preferred_agent: PromptAgentMode,
}

/// SPI 全体で共通のエラー
#[derive(thiserror::Error, Debug)]
pub enum PromptSpiError {
    #[error("context error: {0}")]
    Context(String),
    #[error("build error: {0}")]
    Build(String),
    #[error("parse error: {0}")]
    Parse(String),
}

pub type PromptSpiResult<T> = Result<T, PromptSpiError>;

/// PromptBuilder が実装すべきトレイト
pub trait PromptBuilder: Send + Sync {
    fn metadata(&self) -> PromptMetadata;
    fn build(&self, ctx: PromptContext) -> PromptSpiResult<PromptPayload>;
    fn parse(&self, raw_output: &str) -> PromptSpiResult<PromptParseOutput>;
}

/// プラグインのエントリーポイントで返すファクトリ
pub trait PromptBuilderFactory: Send + Sync {
    fn metadata(&self) -> PromptMetadata;
    fn create(&self) -> Box<dyn PromptBuilder>;
}

/// `extern "C" fn` エントリーポイントの型
#[allow(improper_ctypes_definitions)]
pub type CreatePromptBuilderFactory = unsafe extern "C" fn() -> *mut dyn PromptBuilderFactory;

/// ホスト側ヘルパー：FFI ポインタを Box に戻す
pub unsafe fn factory_from_raw(
    ptr: *mut dyn PromptBuilderFactory,
) -> Box<dyn PromptBuilderFactory> {
    Box::from_raw(ptr)
}

/// プラグイン側ヘルパー：Box を FFI ポインタに変換
pub fn leak_factory(factory: Box<dyn PromptBuilderFactory>) -> *mut dyn PromptBuilderFactory {
    Box::into_raw(factory)
}
