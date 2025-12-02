use crate::conversation_service::ConversationService;
use chat_history::{Conversation, Message, MessageRole};
use gpui::*;
use gpui_component::input::InputState;
use ui_utils::ScrollManager;

/// 会話管理に関する操作
pub struct ConversationActions {
    pub service: ConversationService,
}

impl ConversationActions {
    pub fn new(service: ConversationService) -> Self {
        Self { service }
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
            Message::new(
                MessageRole::System,
                "Welcome to Neko Assistant (LangChain mode enabled)".to_string(),
            )
        } else {
            Message::new(MessageRole::System, "Welcome to Neko Assistant".to_string())
        };
        new_conversation.add_message(welcome_msg);

        self.service
            .replace_conversation(new_conversation)
            .map_err(|e| format!("Failed to save conversation: {}", e))?;

        // 入力フィールドをクリア
        let _ = input_state.update(cx, |view, cx| view.set_value("", window, cx));

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
        self.service
            .save_current()
            .map_err(|e| format!("Failed to save current conversation: {}", e))?;

        self.service
            .load_conversation(conversation_id)
            .map_err(|e| format!("Failed to load conversation: {}", e))?;

        // 入力フィールドをクリア
        let _ = input_state.update(cx, |view, cx| view.set_value("", window, cx));

        // スクロールをリセット
        scroll_manager.mark_scroll_to_bottom();

        Ok(())
    }

    /// 指定IDの会話を削除
    pub fn delete_conversation(&self, conversation_id: &str) -> Result<bool, String> {
        // 現在の会話を削除しようとしている場合はエラー
        if self
            .service
            .current_conversation_id()
            .map(|cur| cur == conversation_id)
            .unwrap_or(false)
        {
            return Err("Cannot delete the currently active conversation".to_string());
        }

        self.service
            .delete_conversation(conversation_id)
            .map_err(|e| format!("Failed to delete conversation: {}", e))?;
        Ok(true)
    }

    /// 会話リストを取得
    pub fn list_conversations(&self) -> Result<Vec<chat_history::ConversationMetadata>, String> {
        self.service
            .list_conversations()
            .map_err(|e| format!("Failed to list conversations: {}", e))
    }
}
