use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use chat_history::{Conversation, Message, MessageRole, ConversationManager};
use crate::mcp_manager::McpManager;

/// メッセージ処理ハンドラー
/// UIから独立して、メッセージの送受信とLLM呼び出しを管理
pub struct MessageHandler {
    conversation: Arc<Mutex<Conversation>>,
    conversation_manager: Arc<Mutex<ConversationManager>>,
    ui_update_tx: mpsc::UnboundedSender<()>,
    use_langchain: bool,
    ollama_url: String,
    model_name: String,
    mcp_manager: Option<Arc<McpManager>>,
}

impl MessageHandler {
    pub fn new(
        conversation: Arc<Mutex<Conversation>>,
        conversation_manager: Arc<Mutex<ConversationManager>>,
        ui_update_tx: mpsc::UnboundedSender<()>,
        use_langchain: bool,
        ollama_url: String,
        model_name: String,
        mcp_manager: Option<Arc<McpManager>>,
    ) -> Self {
        Self {
            conversation,
            conversation_manager,
            ui_update_tx,
            use_langchain,
            ollama_url,
            model_name,
            mcp_manager,
        }
    }

    /// ユーザーメッセージを処理し、AI応答を生成
    pub fn handle_user_message(&self, user_input: String) {
        // ユーザーメッセージを追加
        {
            let mut conv = self.conversation.lock().unwrap();
            conv.add_message(Message::new(MessageRole::User, user_input.clone()));
            
            // タイトルが"New Chat"のままなら、最初のメッセージをタイトルに
            if conv.title == "New Chat" {
                // メッセージを30文字以内に切り詰め
                let title = if user_input.len() > 30 {
                    format!("{}...", &user_input[..30])
                } else {
                    user_input.clone()
                };
                conv.title = title;
            }
            
            // 自動保存
            if let Ok(manager) = self.conversation_manager.lock() {
                let _ = manager.save(&conv);
            }
        }
        let _ = self.ui_update_tx.send(()); // UI更新通知

        if self.use_langchain {
            // LangChainモード - バックグラウンドで非ブロッキング実行
            let conv_bg = self.conversation.clone();
            let manager_bg = self.conversation_manager.clone();
            let ui_tx_bg = self.ui_update_tx.clone();
            let ollama_url = self.ollama_url.clone();
            let model_name = self.model_name.clone();

            // 処理中メッセージを表示
            {
                let mut conv = self.conversation.lock().unwrap();
                conv.add_message(Message::new(MessageRole::System, "Thinking...".to_string()));
            }
            let _ = self.ui_update_tx.send(()); // UI更新通知

            // バックグラウンドスレッドで実行
            tokio::spawn(async move {
                let mut engine = langchain_bridge::LangChainEngine::new(&ollama_url, &model_name);
                match engine.send_message_simple(&user_input).await {
                    Ok(response) => {
                        // "Thinking..."メッセージを削除してレスポンスを追加
                        let mut conv = conv_bg.lock().unwrap();
                        // 最後のメッセージが"Thinking..."なら削除
                        if let Some(last) = conv.messages.last() {
                            if last.content == "Thinking..." && matches!(last.role, MessageRole::System) {
                                conv.messages.pop();
                            }
                        }
                        conv.add_message(Message::new(MessageRole::Assistant, response));
                        // 自動保存
                        if let Ok(manager) = manager_bg.lock() {
                            let _ = manager.save(&conv);
                        }
                        let _ = ui_tx_bg.send(()); // UI更新通知
                    }
                    Err(e) => {
                        let mut conv = conv_bg.lock().unwrap();
                        // "Thinking..."削除
                        if let Some(last) = conv.messages.last() {
                            if last.content == "Thinking..." && matches!(last.role, MessageRole::System) {
                                conv.messages.pop();
                            }
                        }
                        conv.add_message(Message::new(MessageRole::Error, format!("Error: {}", e)));
                        // 自動保存
                        if let Ok(manager) = manager_bg.lock() {
                            let _ = manager.save(&conv);
                        }
                        let _ = ui_tx_bg.send(()); // UI更新通知
                    }
                }
            });
        } else {
            // エコーモード
            let ai_response = format!("(echo) {}", user_input);
            let mut conv = self.conversation.lock().unwrap();
            conv.add_message(Message::new(MessageRole::Assistant, ai_response));
            // 自動保存
            if let Ok(manager) = self.conversation_manager.lock() {
                let _ = manager.save(&conv);
            }
            let _ = self.ui_update_tx.send(()); // UI更新通知
        }
    }
}
