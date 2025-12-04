use chat_core::{ChatState, McpServerStatus};
use chat_history::MessageRole;
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
