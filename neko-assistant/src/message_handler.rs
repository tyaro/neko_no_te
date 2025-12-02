use crate::conversation_service::ConversationService;
use crate::langchain_tools::build_mcp_tools;
use crate::mcp_manager::McpManager;
use chat_history::{Message, MessageRole};
use langchain_bridge::{LangChainEngine, LangChainToolAgent};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex as AsyncMutex};

/// メッセージ処理ハンドラー
/// UIから独立して、メッセージの送受信とLLM呼び出しを管理
pub struct MessageHandler {
    conversation_service: ConversationService,
    ui_update_tx: mpsc::UnboundedSender<()>,
    use_langchain: bool,
    ollama_url: String,
    model_name: String,
    mcp_manager: Option<Arc<McpManager>>,
    langchain_agent: Arc<AsyncMutex<Option<LangChainToolAgent>>>,
}

impl MessageHandler {
    pub fn new(
        conversation_service: ConversationService,
        ui_update_tx: mpsc::UnboundedSender<()>,
        use_langchain: bool,
        ollama_url: String,
        model_name: String,
        mcp_manager: Option<Arc<McpManager>>,
    ) -> Self {
        let langchain_agent = Arc::new(AsyncMutex::new(None));

        if use_langchain {
            if let Some(manager) = mcp_manager.clone() {
                let model = model_name.clone();
                let agent_slot = langchain_agent.clone();
                tokio::spawn(async move {
                    if let Err(e) = ensure_tool_agent(agent_slot, manager, model).await {
                        eprintln!("Failed to initialize MCP tools: {}", e);
                    }
                });
            }
        }

        Self {
            conversation_service,
            ui_update_tx,
            use_langchain,
            ollama_url,
            model_name,
            mcp_manager,
            langchain_agent,
        }
    }

    /// ユーザーメッセージを処理し、AI応答を生成
    pub fn handle_user_message(&self, user_input: String) {
        if let Err(err) = self.record_user_message(&user_input) {
            eprintln!("Failed to record user message: {}", err);
            return;
        }
        let _ = self.ui_update_tx.send(()); // UI更新通知

        if self.use_langchain {
            // LangChainモード - バックグラウンドで非ブロッキング実行
            let service_bg = self.conversation_service.clone();
            let ui_tx_bg = self.ui_update_tx.clone();
            let ollama_url = self.ollama_url.clone();
            let model_name = self.model_name.clone();
            let user_text = user_input.clone();
            let agent_slot = self.langchain_agent.clone();
            let manager = self.mcp_manager.clone();

            // 処理中メッセージを表示
            if let Err(err) = self
                .conversation_service
                .append_message(MessageRole::System, "Thinking...".to_string())
            {
                eprintln!("Failed to append thinking message: {}", err);
            }
            let _ = self.ui_update_tx.send(()); // UI更新通知

            // バックグラウンドスレッドで実行
            tokio::spawn(async move {
                let tool_agent = if let Some(manager) = manager {
                    match ensure_tool_agent(agent_slot.clone(), manager, model_name.clone()).await {
                        Ok(agent) => Some(agent),
                        Err(e) => {
                            eprintln!("Failed to prepare MCP tools: {}", e);
                            None
                        }
                    }
                } else {
                    None
                };

                if let Some(agent) = tool_agent {
                    match agent.invoke(&user_text).await {
                        Ok(response) => {
                            if let Err(err) =
                                finalize_response(&service_bg, MessageRole::Assistant, response)
                            {
                                eprintln!("Failed to record assistant response: {}", err);
                            }
                        }
                        Err(e) => {
                            if let Err(err) = finalize_response(
                                &service_bg,
                                MessageRole::Error,
                                format!("Error: {}", e),
                            ) {
                                eprintln!("Failed to record error response: {}", err);
                            }
                        }
                    }
                } else {
                    let mut engine = LangChainEngine::new(&ollama_url, &model_name);
                    match engine.send_message_simple(&user_text).await {
                        Ok(response) => {
                            if let Err(err) =
                                finalize_response(&service_bg, MessageRole::Assistant, response)
                            {
                                eprintln!("Failed to record assistant response: {}", err);
                            }
                        }
                        Err(e) => {
                            if let Err(err) = finalize_response(
                                &service_bg,
                                MessageRole::Error,
                                format!("Error: {}", e),
                            ) {
                                eprintln!("Failed to record error response: {}", err);
                            }
                        }
                    }
                }

                let _ = ui_tx_bg.send(()); // UI更新通知
            });
        } else {
            // エコーモード
            let ai_response = format!("(echo) {}", user_input);
            if let Err(err) = self
                .conversation_service
                .append_message(MessageRole::Assistant, ai_response)
            {
                eprintln!("Failed to append echo response: {}", err);
            }
            let _ = self.ui_update_tx.send(()); // UI更新通知
        }
    }

    fn record_user_message(&self, user_input: &str) -> chat_history::Result<()> {
        let message_text = user_input.to_string();
        let title_candidate = derive_title(user_input);
        self.conversation_service.mutate_and_save(move |conv| {
            conv.add_message(Message::new(MessageRole::User, message_text.clone()));
            if conv.title == "New Chat" {
                conv.title = title_candidate.clone();
            }
        })
    }
}

fn finalize_response(
    service: &ConversationService,
    role: MessageRole,
    content: String,
) -> chat_history::Result<()> {
    service.pop_last_if(|msg| {
        matches!(msg.role, MessageRole::System) && msg.content == "Thinking..."
    })?;
    service.append_message(role, content)
}

fn derive_title(source: &str) -> String {
    const MAX: usize = 30;
    if source.chars().count() <= MAX {
        return source.to_string();
    }

    let mut truncated = String::with_capacity(MAX + 3);
    for ch in source.chars().take(MAX) {
        truncated.push(ch);
    }
    truncated.push_str("...");
    truncated
}

async fn ensure_tool_agent(
    slot: Arc<AsyncMutex<Option<LangChainToolAgent>>>,
    manager: Arc<McpManager>,
    model: String,
) -> Result<LangChainToolAgent, String> {
    {
        let guard = slot.lock().await;
        if let Some(agent) = guard.as_ref() {
            return Ok(agent.clone());
        }
    }

    let tools = build_mcp_tools(manager.clone()).await?;
    if tools.is_empty() {
        return Err("No MCP tools available".to_string());
    }

    let agent = LangChainToolAgent::new(&model, tools).map_err(|e| e.to_string())?;
    let mut guard = slot.lock().await;
    *guard = Some(agent.clone());
    Ok(agent)
}
