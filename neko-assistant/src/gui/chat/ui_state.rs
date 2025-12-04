use super::data_mappers::ChatStateMapper;
use chat_core::ChatState;
use neko_ui::{ChatMessageRow, ChatSidebarItem, ConsoleLogEntry, McpServerItem, McpToolItem};

#[derive(Clone)]
pub struct ChatUiSnapshot {
    pub sidebar_items: Vec<ChatSidebarItem>,
    pub server_items: Vec<McpServerItem>,
    pub tool_items: Vec<McpToolItem>,
    pub console_logs: Vec<ConsoleLogEntry>,
    pub message_rows: Vec<ChatMessageRow>,
}

impl ChatUiSnapshot {
    pub fn from_state(state: &ChatState) -> Self {
        Self {
            sidebar_items: sidebar_items(state),
            server_items: ChatStateMapper::mcp_server_items(state),
            tool_items: ChatStateMapper::mcp_tool_items(state),
            console_logs: ChatStateMapper::console_log_entries(state),
            message_rows: ChatStateMapper::message_rows(state),
        }
    }
}

fn sidebar_items(state: &ChatState) -> Vec<ChatSidebarItem> {
    let active_id = state.conversation_id.as_deref();
    state
        .conversations
        .iter()
        .map(|meta| ChatSidebarItem {
            id: meta.id.clone(),
            title: meta.title.clone(),
            message_count: meta.message_count,
            active: active_id == Some(meta.id.as_str()),
        })
        .collect()
}

// sidebar 生成は Phase 3 以降も UI ごとに必要となるためコマンドを保持
