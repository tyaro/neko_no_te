use crate::plugins::prompt_builder::{HostPromptBuilderFactory, PromptBuilderRegistry};
use prompt_spi::{
    ConversationRole, DirectiveSource, PromptAgentMode, PromptBuilder, PromptContext,
    PromptExecutionHints, PromptMetadata, PromptParseOutput, PromptPayload, PromptSpiResult,
    ToolInvocation,
};
use serde::Deserialize;
use serde_json::{self, Value};
use std::fmt::Write;

/// Register host-provided prompt builders (used when matching plugins are absent).
pub fn register_builtin_prompt_builders(registry: &mut PromptBuilderRegistry) {
    registry.register_host_builder(
        "qwen3:4b-instruct",
        HostPromptBuilderFactory::new(
            PromptMetadata {
                name: "Builtin Qwen Prompt".into(),
                version: "0.1.0".into(),
                description: Some(
                    "Native prompt builder enabling JSON tool-calling for Qwen3-4B".into(),
                ),
                supported_models: vec!["qwen3:4b-instruct".into()],
                homepage: Some("https://huggingface.co/Qwen/Qwen3-4B-Instruct-2507".into()),
                preferred_agent: PromptAgentMode::DirectProvider,
            },
            PromptAgentMode::DirectProvider,
            50,
            "builtin",
            || Box::new(QwenPromptBuilder::default()),
        ),
    );
}

#[derive(Default)]
struct QwenPromptBuilder;

impl PromptBuilder for QwenPromptBuilder {
    fn metadata(&self) -> PromptMetadata {
        PromptMetadata {
            name: "Builtin Qwen Prompt".into(),
            version: "0.1.0".into(),
            description: Some(
                "Generates structured prompts for Qwen models with tool-calling".into(),
            ),
            supported_models: vec!["qwen3:4b-instruct".into()],
            homepage: Some("https://huggingface.co/Qwen/Qwen3-4B-Instruct-2507".into()),
            preferred_agent: PromptAgentMode::DirectProvider,
        }
    }

    fn build(&self, ctx: PromptContext) -> PromptSpiResult<PromptPayload> {
        let mut prompt = String::new();
        writeln!(
            prompt,
            "You are an assistant that always replies in natural Japanese ({}).",
            ctx.locale
        )
        .ok();
        writeln!(
            prompt,
            "Follow the system directives, use tools when needed, and never hallucinate tool outputs."
        )
        .ok();

        if !ctx.system_directives.is_empty() {
            prompt.push_str("\n# System Directives\n");
            for directive in ctx.system_directives {
                writeln!(
                    prompt,
                    "- [{}] {}",
                    format_directive_source(directive.source),
                    directive.content
                )
                .ok();
            }
        }

        if !ctx.tools.is_empty() {
            prompt.push_str("\n# Available Tools\n");
            for tool in ctx.tools {
                writeln!(
                    prompt,
                    "- {}: {}",
                    tool.name,
                    tool.description.as_deref().unwrap_or("(no description)")
                )
                .ok();
                writeln!(
                    prompt,
                    "  JSON Schema: {}",
                    serde_json::to_string_pretty(&tool.input_schema)
                        .unwrap_or_else(|_| "{}".into())
                )
                .ok();
            }
        } else {
            prompt.push_str(
                "\n# No external tools are available. If the user requests data you cannot access, explain the limitation.\n",
            );
        }

        prompt.push_str("\n# Conversation History\n");
        for turn in ctx.conversation {
            let role = match turn.role {
                ConversationRole::System => "System",
                ConversationRole::User => "User",
                ConversationRole::Assistant => "Assistant",
                ConversationRole::Tool => "Tool",
            };
            writeln!(prompt, "{}: {}", role, turn.content).ok();
            prompt.push_str("---\n");
        }

        prompt.push_str("\n# Response Requirements\n");
        prompt.push_str(
            r#"Return ONLY JSON with this schema:
{
  "tool_requests": [
    {"name": "<tool_name>", "arguments": {<json arguments>}}
  ],
  "final_answer": "<concise Japanese answer>"
}
- If you need more information from a tool, add one or more entries to tool_requests and leave final_answer empty or null.
- When ready to answer, tool_requests must be an empty array and final_answer must contain the reply.
"#,
        );

        Ok(PromptPayload {
            agent_mode: PromptAgentMode::DirectProvider,
            prompt: Some(prompt),
            prompt_variables: Default::default(),
            execution_hints: PromptExecutionHints::default(),
        })
    }

    fn parse(&self, raw_output: &str) -> PromptSpiResult<PromptParseOutput> {
        let json_slice = extract_json_block(raw_output).unwrap_or(raw_output);
        match serde_json::from_str::<QwenResponse>(json_slice) {
            Ok(parsed) => Ok(parsed.into_output()),
            Err(_) => Ok(PromptParseOutput {
                final_answer: Some(raw_output.to_string()),
                tool_requests: Vec::new(),
            }),
        }
    }
}

#[derive(Debug, Deserialize)]
struct QwenResponse {
    #[serde(default)]
    final_answer: Option<String>,
    #[serde(default)]
    tool_requests: Option<Vec<RawToolInvocation>>,
    #[serde(default)]
    tool_calls: Option<Vec<RawToolInvocation>>,
}

impl QwenResponse {
    fn into_output(self) -> PromptParseOutput {
        let mut requests = Vec::new();
        if let Some(mut list) = self.tool_requests {
            requests.append(&mut list);
        }
        if let Some(mut list) = self.tool_calls {
            requests.append(&mut list);
        }

        PromptParseOutput {
            final_answer: self.final_answer,
            tool_requests: requests
                .into_iter()
                .map(|req| ToolInvocation {
                    name: req.name,
                    arguments: req.arguments.unwrap_or(Value::Null),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawToolInvocation {
    name: String,
    arguments: Option<Value>,
}

fn extract_json_block(source: &str) -> Option<&str> {
    let trimmed = source.trim();
    if trimmed.starts_with("```") {
        let trimmed = trimmed
            .trim_start_matches("```json")
            .trim_start_matches("```JSON")
            .trim_start_matches("```");
        let trimmed = trimmed.trim_end_matches("```");
        return Some(trimmed.trim());
    }

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed);
    }

    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if end > start {
            return Some(trimmed[start..=end].trim());
        }
    }

    None
}

fn format_directive_source(source: DirectiveSource) -> &'static str {
    match source {
        DirectiveSource::Host => "host",
        DirectiveSource::User => "user",
        DirectiveSource::Plugin => "plugin",
    }
}
