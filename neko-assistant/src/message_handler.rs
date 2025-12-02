use crate::conversation_service::ConversationService;
use crate::langchain_tools::build_mcp_tools;
use crate::mcp_manager::McpManager;
use crate::plugins::{PromptBuilderRegistry, PromptBuilderSource};
use chat_history::{Message, MessageRole};
use langchain_bridge::{LangChainEngine, LangChainToolAgent};
use model_provider::{ollama_impl::OllamaProvider, ModelProvider};
use prompt_spi::{
    ConversationRole as SpiConversationRole, ConversationTurn as SpiConversationTurn,
    DirectiveSource as SpiDirectiveSource, PromptAgentMode, PromptContext as SpiPromptContext,
    PromptPayload, SystemDirective as SpiSystemDirective, ToolInvocation, ToolSpec as SpiToolSpec,
};
use serde_json;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, Mutex as AsyncMutex};

const DEFAULT_LOCALE: &str = "ja-JP";
const HOST_DIRECTIVE: &str =
    "回答は自然な日本語で丁寧にまとめてください。必要に応じて MCP ツールの結果も含めてください。";

/// メッセージ処理ハンドラー
/// UIから独立して、メッセージの送受信とLLM呼び出しを管理
pub struct MessageHandler {
    conversation_service: ConversationService,
    ui_update_tx: mpsc::UnboundedSender<()>,
    use_langchain: bool,
    ollama_url: String,
    model_name: Arc<Mutex<String>>,
    mcp_manager: Option<Arc<McpManager>>,
    langchain_agent: Arc<AsyncMutex<Option<LangChainToolAgent>>>,
    prompt_registry: Option<Arc<PromptBuilderRegistry>>,
}

impl MessageHandler {
    pub fn new(
        conversation_service: ConversationService,
        ui_update_tx: mpsc::UnboundedSender<()>,
        use_langchain: bool,
        ollama_url: String,
        model_name: String,
        mcp_manager: Option<Arc<McpManager>>,
        prompt_registry: Option<Arc<PromptBuilderRegistry>>,
    ) -> Self {
        let langchain_agent = Arc::new(AsyncMutex::new(None));
        let model_state = Arc::new(Mutex::new(model_name));

        if use_langchain {
            if let Some(manager) = mcp_manager.clone() {
                let agent_slot = langchain_agent.clone();
                let model_snapshot = snapshot_model(&model_state);
                tokio::spawn(async move {
                    if let Err(e) = ensure_tool_agent(agent_slot, manager, model_snapshot).await {
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
            model_name: model_state,
            mcp_manager,
            langchain_agent,
            prompt_registry,
        }
    }

    /// ユーザーメッセージを処理し、AI応答を生成
    pub fn handle_user_message(&self, user_input: String) {
        if let Err(err) = self.record_user_message(&user_input) {
            eprintln!("Failed to record user message: {}", err);
            return;
        }
        let _ = self.ui_update_tx.send(()); // UI更新通知

        let active_model = self.current_model();
        let prompt_builder = self.select_prompt_builder(&active_model);
        let needs_async = prompt_builder.is_some() || self.use_langchain;

        if needs_async {
            if let Err(err) = self
                .conversation_service
                .append_message(MessageRole::System, "Thinking...".to_string())
            {
                eprintln!("Failed to append thinking message: {}", err);
            }
            let _ = self.ui_update_tx.send(()); // UI更新通知
        }

        if let Some(builder_source) = prompt_builder {
            let service_bg = self.conversation_service.clone();
            let ui_tx_bg = self.ui_update_tx.clone();
            let ollama_url = self.ollama_url.clone();
            let model_name = active_model.clone();
            let manager = self.mcp_manager.clone();
            let agent_slot = self.langchain_agent.clone();

            tokio::spawn(async move {
                match run_prompt_builder_session(
                    builder_source,
                    service_bg.clone(),
                    model_name,
                    ollama_url.clone(),
                    manager,
                    agent_slot,
                )
                .await
                {
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

                let _ = ui_tx_bg.send(());
            });

            return;
        }

        if self.use_langchain {
            // LangChainモード - バックグラウンドで非ブロッキング実行
            let service_bg = self.conversation_service.clone();
            let ui_tx_bg = self.ui_update_tx.clone();
            let ollama_url = self.ollama_url.clone();
            let model_name = active_model.clone();
            let user_text = user_input.clone();
            let agent_slot = self.langchain_agent.clone();
            let manager = self.mcp_manager.clone();

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

    pub fn set_model(&self, new_model: String) -> Result<(), String> {
        {
            let mut guard = self
                .model_name
                .lock()
                .map_err(|_| "Failed to lock model state".to_string())?;
            if *guard == new_model {
                return Ok(());
            }
            *guard = new_model.clone();
        }

        if self.use_langchain {
            if let Some(manager) = self.mcp_manager.clone() {
                let agent_slot = self.langchain_agent.clone();
                let agent_slot_for_init = agent_slot.clone();
                tokio::spawn(async move {
                    {
                        let mut guard = agent_slot.lock().await;
                        *guard = None;
                    }
                    if let Err(e) = ensure_tool_agent(agent_slot_for_init, manager, new_model).await
                    {
                        eprintln!("Failed to reinitialize MCP tools: {}", e);
                    }
                });
            }
        }

        Ok(())
    }

    fn current_model(&self) -> String {
        snapshot_model(&self.model_name)
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

    fn select_prompt_builder(&self, model: &str) -> Option<PromptBuilderSource> {
        self.prompt_registry
            .as_ref()
            .and_then(|registry| registry.resolve(model))
    }
}

async fn run_prompt_builder_session(
    source: PromptBuilderSource,
    service: ConversationService,
    model_name: String,
    ollama_url: String,
    manager: Option<Arc<McpManager>>,
    agent_slot: Arc<AsyncMutex<Option<LangChainToolAgent>>>,
) -> Result<String, String> {
    let builder = source.create_builder();

    let tool_specs = collect_tool_specs(manager.clone()).await?;

    let conversation = service.snapshot().map_err(|e| e.to_string())?;
    let mut roles: Vec<SpiConversationRole> = Vec::with_capacity(conversation.messages.len());
    let mut message_storage: Vec<String> = Vec::with_capacity(conversation.messages.len());

    for message in conversation.messages.into_iter() {
        if should_skip_placeholder(&message) {
            continue;
        }

        if let Some(role) = map_message_role(message.role) {
            roles.push(role);
            message_storage.push(message.content);
        }
    }

    let mut conversation_turns: Vec<SpiConversationTurn> = Vec::with_capacity(roles.len());
    for (idx, role) in roles.iter().enumerate() {
        conversation_turns.push(SpiConversationTurn {
            role: *role,
            content: &message_storage[idx],
        });
    }

    let system_directives = vec![SpiSystemDirective {
        source: SpiDirectiveSource::Host,
        content: HOST_DIRECTIVE,
    }];

    let context = SpiPromptContext {
        model: &model_name,
        locale: DEFAULT_LOCALE,
        conversation: &conversation_turns,
        tools: &tool_specs,
        system_directives: &system_directives,
    };

    let payload = builder
        .build(context)
        .map_err(|e| format!("Prompt build error: {}", e))?;

    let raw_output = match payload.agent_mode {
        PromptAgentMode::LangChain => {
            execute_with_langchain(
                &payload,
                agent_slot.clone(),
                manager.clone(),
                model_name.clone(),
                ollama_url.clone(),
            )
            .await?
        }
        PromptAgentMode::DirectProvider => {
            execute_direct_provider(&payload, &ollama_url, &model_name).await?
        }
    };

    let parsed = builder
        .parse(&raw_output)
        .map_err(|e| format!("Prompt parse error: {}", e))?;

    if !parsed.tool_requests.is_empty() {
        let tool_text = fulfill_prompt_builder_tools(parsed.tool_requests, manager.clone()).await?;
        if let Some(answer) = parsed.final_answer {
            if answer.trim().is_empty() {
                return Ok(tool_text);
            }
            return Ok(format!("{}\n\n{}", answer.trim(), tool_text));
        }
        return Ok(tool_text);
    }

    if let Some(answer) = parsed.final_answer {
        return Ok(answer);
    }

    Ok(raw_output)
}

async fn execute_direct_provider(
    payload: &PromptPayload,
    ollama_url: &str,
    model_name: &str,
) -> Result<String, String> {
    let prompt_text = extract_prompt(payload)?;
    let provider = OllamaProvider::new(ollama_url)
        .map_err(|e| format!("Invalid Ollama URL '{}': {}", ollama_url, e))?;

    provider
        .generate(model_name, &prompt_text)
        .await
        .map(|result| result.text)
        .map_err(|e| format!("Direct provider error: {}", e))
}

async fn execute_with_langchain(
    payload: &PromptPayload,
    agent_slot: Arc<AsyncMutex<Option<LangChainToolAgent>>>,
    manager: Option<Arc<McpManager>>,
    model_name: String,
    ollama_url: String,
) -> Result<String, String> {
    let prompt_text = extract_prompt(payload)?;

    if let Some(manager) = manager {
        match ensure_tool_agent(agent_slot.clone(), manager.clone(), model_name.clone()).await {
            Ok(agent) => {
                return agent
                    .invoke(&prompt_text)
                    .await
                    .map_err(|e| format!("LangChain agent error: {}", e));
            }
            Err(err) => {
                eprintln!("Falling back to plain LLM: {}", err);
            }
        }
    }

    let mut engine = LangChainEngine::new(&ollama_url, &model_name);
    engine
        .send_message_simple(&prompt_text)
        .await
        .map_err(|e| format!("LangChain engine error: {}", e))
}

async fn collect_tool_specs(manager: Option<Arc<McpManager>>) -> Result<Vec<SpiToolSpec>, String> {
    let mut specs = Vec::new();
    let Some(manager) = manager else {
        return Ok(specs);
    };

    for (server, tool) in manager.get_all_tools().await? {
        specs.push(SpiToolSpec {
            name: format!("{}@{}", tool.name, server),
            description: Some(format!("{} (server: {})", tool.description, server)),
            input_schema: tool.input_schema,
        });
    }

    Ok(specs)
}

async fn fulfill_prompt_builder_tools(
    requests: Vec<ToolInvocation>,
    manager: Option<Arc<McpManager>>,
) -> Result<String, String> {
    let manager = manager.ok_or_else(|| {
        "Prompt builder requested tool calls, but no MCP servers are configured".to_string()
    })?;

    if requests.is_empty() {
        return Err("No tool requests provided".to_string());
    }

    let mut outputs = Vec::new();
    for invocation in requests {
        let (tool_name, server_name) = split_tool_identifier(&invocation.name)?;
        let arguments = invocation.arguments;
        match manager.call_tool(&server_name, &tool_name, arguments).await {
            Ok(result) => {
                let pretty =
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string());
                outputs.push(format!(
                    "ツール: {} @ {}\n{}",
                    tool_name, server_name, pretty
                ));
            }
            Err(err) => outputs.push(format!(
                "ツール: {} @ {} (エラー)\n{}",
                tool_name, server_name, err
            )),
        }
    }

    Ok(format!(
        "以下の MCP ツールを実行しました:\n\n{}",
        outputs.join("\n\n")
    ))
}

fn split_tool_identifier(identifier: &str) -> Result<(String, String), String> {
    let mut parts = identifier.rsplitn(2, '@');
    let server = parts
        .next()
        .ok_or_else(|| format!("Invalid tool identifier: {}", identifier))?;
    let tool = parts
        .next()
        .ok_or_else(|| format!("Missing tool name in identifier: {}", identifier))?;
    Ok((tool.to_string(), server.to_string()))
}

fn extract_prompt(payload: &PromptPayload) -> Result<String, String> {
    payload
        .prompt
        .as_ref()
        .map(|text| text.clone())
        .ok_or_else(|| "Prompt builder did not provide a prompt payload".to_string())
}

fn should_skip_placeholder(message: &Message) -> bool {
    matches!(message.role, MessageRole::System) && message.content == "Thinking..."
}

fn map_message_role(role: MessageRole) -> Option<SpiConversationRole> {
    match role {
        MessageRole::User => Some(SpiConversationRole::User),
        MessageRole::Assistant => Some(SpiConversationRole::Assistant),
        MessageRole::System => Some(SpiConversationRole::System),
        MessageRole::Error => Some(SpiConversationRole::Assistant),
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

fn snapshot_model(state: &Arc<Mutex<String>>) -> String {
    match state.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}
