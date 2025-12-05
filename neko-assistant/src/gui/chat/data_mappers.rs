use chat_core::{ChatState, McpServerStatus};
use chat_history::{Message, MessageRole};
use neko_ui::{
    ChatMessageRow, ConsoleLogEntry, McpServerItem, McpServerStatusBadge, McpToolItem, MessageType,
};

pub struct ChatStateMapper;

impl ChatStateMapper {
    pub fn message_rows(state: &ChatState) -> Vec<ChatMessageRow> {
        state
            .messages
            .iter()
            .map(|msg| ChatMessageRow {
                content: msg.content.clone(),
                message_type: match msg.role {
                    MessageRole::User => MessageType::User,
                    MessageRole::Assistant => MessageType::Assistant,
                    MessageRole::System => MessageType::System,
                    MessageRole::Error => MessageType::Error,
                },
                align_end: matches!(msg.role, MessageRole::User),
                is_thinking: is_thinking_message(msg),
                source_label: message_source_label(msg),
            })
            .collect()
    }

    pub fn mcp_server_items(state: &ChatState) -> Vec<McpServerItem> {
        state
            .mcp_servers
            .iter()
            .map(|server| McpServerItem {
                name: server.name.clone(),
                status: match server.status {
                    McpServerStatus::Unknown => McpServerStatusBadge::Unknown,
                    McpServerStatus::Ready => McpServerStatusBadge::Ready,
                    McpServerStatus::Error(_) => McpServerStatusBadge::Error,
                },
                tool_count: server.tool_count,
                message: match &server.status {
                    McpServerStatus::Error(msg) => Some(msg.clone()),
                    _ => None,
                },
            })
            .collect()
    }

    pub fn mcp_tool_items(state: &ChatState) -> Vec<McpToolItem> {
        state
            .mcp_tools
            .iter()
            .map(|tool| McpToolItem {
                server_name: tool.server_name.clone(),
                tool_name: tool.tool_name.clone(),
                description: tool.description.clone(),
            })
            .collect()
    }

    pub fn console_log_entries(state: &ChatState) -> Vec<ConsoleLogEntry> {
        state
            .console_logs
            .iter()
            .map(|record| ConsoleLogEntry {
                role_label: match record.kind {
                    chat_core::ConsoleLogKind::Input => "Input".to_string(),
                    chat_core::ConsoleLogKind::Output => "Output".to_string(),
                    chat_core::ConsoleLogKind::Error => "Error".to_string(),
                },
                content: record.content.clone(),
            })
            .collect()
    }
}

fn is_thinking_message(message: &Message) -> bool {
    message
        .metadata
        .as_ref()
        .and_then(|meta| meta.get("thinking"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

fn message_source_label(message: &Message) -> Option<String> {
    let metadata = message.metadata.as_ref()?;
    let source = metadata.get("source")?.as_str()?;

    if source != "mcp" {
        return None;
    }

    let origin = metadata
        .get("origin")
        .and_then(|value| value.as_str())
        .unwrap_or("");

    let label = match origin {
        "prompt_builder" => "MCP (Prompt Builder)",
        "langchain_chat" => "MCP (LangChain)",
        _ => "MCP",
    };

    Some(label.to_string())
}
