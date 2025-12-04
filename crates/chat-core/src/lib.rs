pub mod chat_controller;
mod conversation_service;

pub mod console_log;
pub mod langchain_tools;
pub mod mcp_client;
pub mod mcp_manager;
pub mod message_handler;
pub mod plugins;
pub mod prompt_builders;

pub use chat_controller::{
    ChatCommand, ChatController, ChatControllerConfig, ChatEvent, ChatState, ControllerError,
    ControllerSubscription, McpServerMetadata, McpServerStatus, McpToolMetadata,
};
pub use console_log::{ConsoleLogKind, ConsoleLogRecord};
pub use conversation_service::ConversationService;
pub use mcp_client::{
    create_sample_config, load_mcp_config, save_mcp_config, McpClient, McpServerConfig, McpTool,
};
pub use mcp_manager::McpManager;
pub use message_handler::MessageHandler;
pub use plugins::{
    disable_plugin, discover_plugins, enable_plugin,
    metadata::PluginEntry,
    prompt_builder::{HostPromptBuilderFactory, PromptBuilderRegistry, PromptBuilderSource},
};
pub use prompt_builders::register_builtin_prompt_builders;
