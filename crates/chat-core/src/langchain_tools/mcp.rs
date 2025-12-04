use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use langchain_rust::tools::Tool;
use serde_json::{json, Value};

use crate::mcp_client::McpTool;
use crate::mcp_manager::McpManager;

/// LangChain 用の MCP ツール一覧を構築
pub async fn build_mcp_tools(
    manager: Arc<McpManager>,
    on_tool_used: Option<Arc<dyn Fn() + Send + Sync>>,
) -> Result<Vec<Arc<dyn Tool>>, String> {
    let specs = manager.get_all_tools().await?;
    let mut tools: Vec<Arc<dyn Tool>> = Vec::new();

    for (server_name, spec) in specs {
        let tool = McpLangChainTool::new(
            manager.clone(),
            server_name,
            spec,
            on_tool_used.as_ref().map(Arc::clone),
        );
        tools.push(Arc::new(tool) as Arc<dyn Tool>);
    }

    Ok(tools)
}

/// LangChain の Tool トレイトへ MCP ツールをブリッジ
pub struct McpLangChainTool {
    manager: Arc<McpManager>,
    server_name: String,
    tool_name: String,
    display_name: String,
    description: String,
    input_schema: Value,
    on_tool_used: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl McpLangChainTool {
    pub fn new(
        manager: Arc<McpManager>,
        server_name: String,
        spec: McpTool,
        on_tool_used: Option<Arc<dyn Fn() + Send + Sync>>,
    ) -> Self {
        let display_name = spec.name.clone();
        let description = format!("{} (MCP server: {})", spec.description, server_name);

        Self {
            manager,
            server_name,
            tool_name: spec.name,
            display_name,
            description,
            input_schema: spec.input_schema,
            on_tool_used,
        }
    }
}

#[async_trait]
impl Tool for McpLangChainTool {
    fn name(&self) -> String {
        self.display_name.clone()
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn parameters(&self) -> Value {
        if self.input_schema.is_object() {
            self.input_schema.clone()
        } else {
            json!({
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": self.description(),
                    }
                },
                "required": ["input"],
            })
        }
    }

    async fn run(&self, input: Value) -> Result<String, Box<dyn std::error::Error>> {
        let arguments = normalize_arguments(input, &self.input_schema);
        let result = self
            .manager
            .call_tool(&self.server_name, &self.tool_name, arguments)
            .await;

        if let Some(callback) = self.on_tool_used.as_ref().map(Arc::clone) {
            callback();
        }

        let response = result.map_err(|e| anyhow!(e))?;

        Ok(render_result(response))
    }
}

fn normalize_arguments(input: Value, schema: &Value) -> Value {
    match input {
        Value::String(text) => {
            if let Ok(parsed) = serde_json::from_str(&text) {
                return parsed;
            }

            if let Some(prop) = infer_primary_property(schema) {
                return json!({ prop: text });
            }

            Value::String(text)
        }
        other => other,
    }
}

fn infer_primary_property(schema: &Value) -> Option<String> {
    let obj = schema.as_object()?;
    let schema_type = obj
        .get("type")
        .and_then(|v| v.as_str())
        .map(|s| s.eq_ignore_ascii_case("object"))
        .unwrap_or(false);
    if !schema_type {
        return None;
    }

    if let Some(any_of) = obj.get("anyOf").and_then(|v| v.as_array()) {
        for variant in any_of {
            if let Some(required) = variant.get("required").and_then(|v| v.as_array()) {
                if let Some(first) = required.iter().filter_map(|v| v.as_str()).next() {
                    return Some(first.to_string());
                }
            }
        }
    }

    if let Some(required) = obj.get("required").and_then(|v| v.as_array()) {
        if let Some(first) = required.iter().filter_map(|v| v.as_str()).next() {
            return Some(first.to_string());
        }
    }

    if let Some(properties) = obj.get("properties").and_then(|v| v.as_object()) {
        if properties.len() == 1 {
            if let Some((name, _)) = properties.iter().next() {
                return Some(name.clone());
            }
        }

        for candidate in ["input", "query", "text", "prompt"] {
            if properties.contains_key(candidate) {
                return Some(candidate.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_string_using_anyof_required_field() {
        let schema = json!({
            "type": "object",
            "anyOf": [
                {"required": ["city"]},
                {"required": ["city_code"]}
            ],
            "properties": {
                "city": {"type": "string"},
                "city_code": {"type": "string"}
            }
        });

        let input = Value::String("Osaka".to_string());
        let normalized = normalize_arguments(input, &schema);
        assert_eq!(normalized, json!({"city": "Osaka"}));
    }

    #[test]
    fn keeps_string_when_no_schema_hint() {
        let schema = json!({
            "type": "string"
        });

        let input = Value::String("raw".to_string());
        let normalized = normalize_arguments(input.clone(), &schema);
        assert_eq!(normalized, input);
    }

    #[test]
    fn prefers_single_property_object() {
        let schema = json!({
            "type": "object",
            "properties": {
                "query": {"type": "string"}
            }
        });

        let input = Value::String("search term".to_string());
        let normalized = normalize_arguments(input, &schema);
        assert_eq!(normalized, json!({"query": "search term"}));
    }
}

fn render_result(value: Value) -> String {
    match value {
        Value::String(s) => s,
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(_) | Value::Object(_) => {
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
        }
    }
}
