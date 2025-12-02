use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use langchain_rust::tools::Tool;
use serde_json::{json, Value};

use crate::mcp_client::McpTool;
use crate::mcp_manager::McpManager;

/// LangChain 用の MCP ツール一覧を構築
pub async fn build_mcp_tools(manager: Arc<McpManager>) -> Result<Vec<Arc<dyn Tool>>, String> {
    let specs = manager.get_all_tools().await?;
    let mut tools: Vec<Arc<dyn Tool>> = Vec::new();

    for (server_name, spec) in specs {
        let tool = McpLangChainTool::new(manager.clone(), server_name, spec);
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
}

impl McpLangChainTool {
    pub fn new(manager: Arc<McpManager>, server_name: String, spec: McpTool) -> Self {
        let display_name = format!("{}@{}", spec.name, server_name);
        let description = format!("{} (server: {})", spec.description, server_name);

        Self {
            manager,
            server_name,
            tool_name: spec.name,
            display_name,
            description,
            input_schema: spec.input_schema,
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
        let arguments = normalize_arguments(input);
        let response = self
            .manager
            .call_tool(&self.server_name, &self.tool_name, arguments)
            .await
            .map_err(|e| anyhow!(e))?;

        Ok(render_result(response))
    }
}

fn normalize_arguments(input: Value) -> Value {
    match input {
        Value::String(text) => serde_json::from_str(&text).unwrap_or(Value::String(text)),
        other => other,
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
