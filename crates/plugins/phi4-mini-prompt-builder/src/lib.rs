//! Phi-4-mini prompt builder plugin (PromptBuilder SPI)
//!
//! Produces prompts compatible with the existing `phi4-mini-adapter` format
//! (<|system|>, <|user|>, <|assistant|> and <|tool|> blocks) and implements
//! a basic parse method that returns the entire model text as the final
//! answer when no tool calls are detected.

use prompt_spi::*;
use serde_json::json;
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// The plugin's runtime PromptBuilder implementation.
struct Phi4PromptBuilder;

impl Phi4PromptBuilder {
    fn format_tools(tools: &[ToolSpec]) -> String {
        let tool_defs: Vec<_> = tools
            .iter()
            .map(|tool| {
                json!({
                    "name": tool.name,
                    "description": tool.description.as_deref().unwrap_or(""),
                    "parameters": tool.input_schema.clone(),
                })
            })
            .collect();

        format!("<|tool|>\n{}\n<|/tool|>", serde_json::to_string_pretty(&tool_defs).unwrap())
    }
}

impl PromptBuilder for Phi4PromptBuilder {
    fn metadata(&self) -> PromptMetadata {
        PromptMetadata {
            name: "phi4-mini-prompt-builder".to_string(),
            version: "0.1.0".to_string(),
            description: Some("phi4-mini instructed prompt builder (plugin)".to_string()),
            supported_models: vec![
                "phi4-mini:3.8b".to_string(),
                "Phi-4-mini-instruct".to_string(),
            ],
            homepage: None,
            preferred_agent: PromptAgentMode::DirectProvider,
        }
    }

    fn build(&self, ctx: PromptContext) -> PromptSpiResult<PromptPayload> {
        // Build a phi4-mini style prompt. Include system directives + tools if provided.
        let mut system = String::new();

        if !ctx.system_directives.is_empty() {
            // join system directives into a single system string.
            for d in ctx.system_directives {
                let src = match d.source {
                    DirectiveSource::Host => "[host]",
                    DirectiveSource::User => "[user]",
                    DirectiveSource::Plugin => "[plugin]",
                };
                system.push_str(&format!("{} {}\n", src, d.content));
            }
        }

        if system.trim().is_empty() {
            system.push_str("You are a helpful assistant.");
        }

        if !ctx.tools.is_empty() {
            system.push_str(" with access to these tools.\n");
            system.push_str(&Self::format_tools(ctx.tools));
        }

        let mut prompt = String::new();
        prompt.push_str("<|system|>\n");
        prompt.push_str(&system);
        prompt.push_str("<|end|>\n\n");

        // Build conversation content. Use the conversation slices from PromptContext.
        for turn in ctx.conversation {
            match turn.role {
                ConversationRole::System => {
                    prompt.push_str("<|system|>\n");
                    prompt.push_str(turn.content);
                    prompt.push_str("<|end|>\n\n");
                }
                ConversationRole::User => {
                    prompt.push_str("<|user|>\n");
                    prompt.push_str(turn.content);
                    prompt.push_str("<|end|>\n\n");
                }
                ConversationRole::Assistant => {
                    prompt.push_str("<|assistant|>\n");
                    prompt.push_str(turn.content);
                    prompt.push_str("<|end|>\n\n");
                }
                ConversationRole::Tool => {
                    // Tools inside the conversation may be rendered as-is.
                    prompt.push_str("<|tool|>\n");
                    prompt.push_str(turn.content);
                    prompt.push_str("<|/tool|>\n\n");
                }
            }
        }

        // When asking the model for a new assistant response, include an assistant open token
        prompt.push_str("<|assistant|>\n");

        Ok(PromptPayload::with_prompt(prompt, PromptAgentMode::DirectProvider))
    }

    fn parse(&self, raw_output: &str) -> PromptSpiResult<PromptParseOutput> {
        // Very small parser: detect JSON tool invocation blocks like
        // "<|tool-invoke|>{...}<|/tool-invoke|>" or otherwise return final_answer
        let mut output = PromptParseOutput::default();

        // Look for a very naive tool invocation marker in the output for now.
        if raw_output.contains("<|tool-invoke|") {
            // Attempt to extract JSON payload between tags
            let start_tag = "<|tool-invoke|>";
            let end_tag = "<|/tool-invoke|>";
            let mut i = 0usize;
            while let Some(idx) = raw_output[i..].find(start_tag) {
                let idx = i + idx + start_tag.len();
                if let Some(end_idx_rel) = raw_output[idx..].find(end_tag) {
                    let end_idx = idx + end_idx_rel;
                    let fragment = raw_output[idx..end_idx].trim();
                    if let Ok(v) = serde_json::from_str::<Value>(fragment) {
                        // Expect { "name": "toolname", "arguments": { ... } }
                        if let Some(name) = v.get("name").and_then(|s| s.as_str()) {
                            let args = v.get("arguments").cloned().unwrap_or_else(|| json!({}));
                            output.tool_requests.push(ToolInvocation {
                                name: name.to_string(),
                                arguments: args,
                            });
                        }
                    }
                    i = end_idx + end_tag.len();
                    continue;
                } else {
                    break;
                }
            }

            // If we found tool requests, do not set final_answer here.
            Ok(output)
        } else {
            // No tool invocation detected, treat whole output as final answer.
            output.final_answer = Some(raw_output.to_string());
            Ok(output)
        }
    }
}

/// Factory that creates the prompt builder.
struct Phi4Factory;

impl Phi4Factory {
    pub fn new() -> Self {
        Self {}
    }
}

impl PromptBuilderFactory for Phi4Factory {
    fn metadata(&self) -> PromptMetadata {
        Phi4PromptBuilder.metadata()
    }

    fn create(&self) -> Box<dyn PromptBuilder> {
        Box::new(Phi4PromptBuilder)
    }
}

#[no_mangle]
pub unsafe extern "C" fn create_prompt_builder() -> *mut dyn PromptBuilderFactory {
    // Leak the factory as an FFI pointer the host will take ownership of.
    leak_factory(Box::new(Phi4Factory::new()))
}

/// JSON-based FFI helper for external runners. Accepts a JSON-serialized
/// representation of a minimal PromptContext and returns a JSON-serialized
/// PromptPayload (owned string). The returned pointer must be freed by the
/// caller using the standard C free (we allocate via CString::into_raw).
#[no_mangle]
pub unsafe extern "C" fn build_prompt_json(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }

    let js = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    #[derive(serde::Deserialize)]
    struct OwnedTurn {
        role: Option<String>,
        content: String,
    }

    #[derive(serde::Deserialize)]
    struct OwnedCtx {
        model: String,
        locale: Option<String>,
        conversation: Option<Vec<OwnedTurn>>,
        // not supporting tools/system_directives in this thin FFI
    }

    let ctx: OwnedCtx = match serde_json::from_str(js) {
        Ok(v) => v,
        Err(_) => return std::ptr::null_mut(),
    };

    // Convert to PromptContext with temporary owned storage
    let conv_owned: Vec<String> = ctx
        .conversation
        .as_ref()
        .map(|vec| vec.iter().map(|t| t.content.clone()).collect())
        .unwrap_or_default();

    // Build ConversationTurn slice referencing the owned strings above.
    let mut conv_turns = Vec::new();
    if let Some(v) = ctx.conversation.as_ref() {
        for (i, t) in v.iter().enumerate() {
            let role = match t.role.as_deref().unwrap_or("user") {
                "system" => prompt_spi::ConversationRole::System,
                "assistant" => prompt_spi::ConversationRole::Assistant,
                "tool" => prompt_spi::ConversationRole::Tool,
                _ => prompt_spi::ConversationRole::User,
            };
            // safe: conv_owned[i] lives until after build is called
            conv_turns.push(prompt_spi::ConversationTurn { role, content: &conv_owned[i] });
        }
    }

    let prompt_ctx = prompt_spi::PromptContext {
        model: &ctx.model,
        locale: ctx.locale.as_deref().unwrap_or("en-US"),
        conversation: &conv_turns,
        tools: &[],
        system_directives: &[],
    };

    let builder = Phi4PromptBuilder;
    match builder.build(prompt_ctx) {
        Ok(payload) => match serde_json::to_string(&payload) {
            Ok(s) => {
                let c = CString::new(s).unwrap_or_default();
                c.into_raw()
            }
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_phi4_prompt() {
        let b = Phi4PromptBuilder;

        let conv = [ConversationTurn { role: ConversationRole::User, content: "こんにちは" }];
        let ctx = PromptContext {
            model: "phi4-mini:3.8b",
            locale: "ja-JP",
            conversation: &conv,
            tools: &[],
            system_directives: &[],
        };

        let payload = b.build(ctx).expect("build ok");
        let p = payload.prompt.expect("prompt present");
        assert!(p.contains("<|system|>"));
        assert!(p.contains("<|user|>"));
        assert!(p.contains("<|assistant|>"));
    }

    #[test]
    fn parse_returns_final_answer() {
        let b = Phi4PromptBuilder;
        let out = "これは返答です。";
        let r = b.parse(out).unwrap();
        assert_eq!(r.final_answer.unwrap(), out);
        assert!(r.tool_requests.is_empty());
    }
}
