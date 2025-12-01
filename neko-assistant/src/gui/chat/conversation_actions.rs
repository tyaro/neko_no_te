use gpui::*;
use std::sync::{Arc, Mutex};
use chat_history::{Conversation, Message, MessageRole, ConversationManager};
use gpui_component::input::InputState;
use ui_utils::ScrollManager;

/// 会話管理に関する操作
pub struct ConversationActions {
    pub conversation: Arc<Mutex<Conversation>>,
    pub conversation_manager: Arc<Mutex<ConversationManager>>,
}

impl ConversationActions {
    pub fn new(
        conversation: Arc<Mutex<Conversation>>,
        conversation_manager: Arc<Mutex<ConversationManager>>,
    ) -> Self {
        Self {
            conversation,
            conversation_manager,
        }
    }

    /// 新規会話を作成して切り替え
    pub fn create_new_conversation(
        &self,
        input_state: &Entity<InputState>,
        scroll_manager: &mut ScrollManager,
        window: &mut Window,
        cx: &mut Context<impl Render>,
    ) -> Result<(), String> {
        // 設定を読み込み
        let config = app_config::AppConfig::load_or_default();
        let use_langchain = config.use_langchain;

        // 新しい会話を作成
        let mut new_conversation = Conversation::new("New Chat");
        let welcome_msg = if use_langchain {
            Message::new(MessageRole::System, "Welcome to Neko Assistant (LangChain mode enabled)".to_string())
        } else {
            Message::new(MessageRole::System, "Welcome to Neko Assistant".to_string())
        };
        new_conversation.add_message(welcome_msg);

        // 保存
        if let Ok(manager) = self.conversation_manager.lock() {
            manager.save(&new_conversation)
                .map_err(|e| format!("Failed to save conversation: {}", e))?;
        }

        // 会話を切り替え
        if let Ok(mut conv) = self.conversation.lock() {
            *conv = new_conversation;
        }

        // 入力フィールドをクリア
        let _ = input_state.update(cx, |view, cx| {
            view.set_value("", window, cx)
        });

        // スクロールをリセット
        scroll_manager.mark_scroll_to_bottom();

        Ok(())
    }

    /// 指定IDの会話に切り替え
    pub fn switch_conversation(
        &self,
        conversation_id: &str,
        input_state: &Entity<InputState>,
        scroll_manager: &mut ScrollManager,
        window: &mut Window,
        cx: &mut Context<impl Render>,
    ) -> Result<(), String> {
        // 現在の会話を保存
        if let Ok(current_conv) = self.conversation.lock() {
            if let Ok(manager) = self.conversation_manager.lock() {
                manager.save(&current_conv)
                    .map_err(|e| format!("Failed to save current conversation: {}", e))?;
            }
        }

        // 新しい会話をロード
        if let Ok(manager) = self.conversation_manager.lock() {
            let loaded_conv = manager.load(conversation_id)
                .map_err(|e| format!("Failed to load conversation: {}", e))?;
            
            if let Ok(mut conv) = self.conversation.lock() {
                *conv = loaded_conv;
            }

            // 入力フィールドをクリア
            let _ = input_state.update(cx, |view, cx| {
                view.set_value("", window, cx)
            });

            // スクロールをリセット
            scroll_manager.mark_scroll_to_bottom();

            Ok(())
        } else {
            Err("Failed to lock conversation manager".to_string())
        }
    }

    /// 指定IDの会話を削除
    pub fn delete_conversation(
        &self,
        conversation_id: &str,
        current_conversation_id: &str,
    ) -> Result<bool, String> {
        // 現在の会話を削除しようとしている場合はエラー
        if conversation_id == current_conversation_id {
            return Err("Cannot delete the currently active conversation".to_string());
        }

        if let Ok(manager) = self.conversation_manager.lock() {
            manager.delete(conversation_id)
                .map_err(|e| format!("Failed to delete conversation: {}", e))?;
            Ok(true)
        } else {
            Err("Failed to lock conversation manager".to_string())
        }
    }

    /// 会話リストを取得
    pub fn list_conversations(&self) -> Result<Vec<chat_history::ConversationMetadata>, String> {
        if let Ok(manager) = self.conversation_manager.lock() {
            manager.list_metadata()
                .map_err(|e| format!("Failed to list conversations: {}", e))
        } else {
            Err("Failed to lock conversation manager".to_string())
        }
    }
}
