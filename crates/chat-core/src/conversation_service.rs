use std::sync::{Arc, Mutex, MutexGuard};

use chat_history::{
    Conversation, ConversationManager, ConversationMetadata, HistoryError, Message, MessageRole,
    Result as HistoryResult,
};

/// 会話データと永続化を仲介するサービス層。
/// UI やハンドラーが直接 Mutex を触らずに済むよう共通処理をまとめる。
#[derive(Clone)]
pub struct ConversationService {
    conversation: Arc<Mutex<Conversation>>,
    manager: Arc<Mutex<ConversationManager>>,
}

impl ConversationService {
    pub fn new(
        conversation: Arc<Mutex<Conversation>>,
        manager: Arc<Mutex<ConversationManager>>,
    ) -> Self {
        Self {
            conversation,
            manager,
        }
    }

    /// 現在の会話スナップショットを取得。
    pub fn snapshot(&self) -> HistoryResult<Conversation> {
        let conv = self.conversation_guard()?;
        Ok(conv.clone())
    }

    /// 現在の会話ID（ロックが取得できない場合は None）。
    pub fn current_conversation_id(&self) -> Option<String> {
        self.snapshot().map(|conv| conv.id).ok()
    }

    /// メッセージ一覧（失敗時は空）。
    pub fn current_messages(&self) -> Vec<Message> {
        self.snapshot()
            .map(|conv| conv.messages)
            .unwrap_or_default()
    }

    /// 会話メタデータ一覧。
    pub fn list_conversations(&self) -> HistoryResult<Vec<ConversationMetadata>> {
        let manager = self.manager_guard()?;
        manager.list_metadata()
    }

    /// 会話を永続化。
    pub fn save_current(&self) -> HistoryResult<()> {
        let snapshot = self.snapshot()?;
        let manager = self.manager_guard()?;
        manager.save(&snapshot)
    }

    /// 指定 ID の会話へ切り替え。
    pub fn load_conversation(&self, conversation_id: &str) -> HistoryResult<()> {
        let conversation = {
            let manager = self.manager_guard()?;
            manager.load(conversation_id)?
        };

        {
            let mut conv = self.conversation_guard_mut()?;
            *conv = conversation;
        }

        Ok(())
    }

    /// 現在の会話を新しい内容で置き換え保存。
    pub fn replace_conversation(&self, conversation: Conversation) -> HistoryResult<()> {
        {
            let mut conv = self.conversation_guard_mut()?;
            *conv = conversation.clone();
        }

        let manager = self.manager_guard()?;
        manager.save(&conversation)
    }

    /// 会話を削除。
    pub fn delete_conversation(&self, conversation_id: &str) -> HistoryResult<()> {
        let manager = self.manager_guard()?;
        manager.delete(conversation_id)
    }

    /// 会話をまとめて更新し保存。
    pub fn mutate_and_save<F>(&self, mutator: F) -> HistoryResult<()>
    where
        F: FnOnce(&mut Conversation),
    {
        {
            let mut conv = self.conversation_guard_mut()?;
            mutator(&mut conv);
        }
        self.save_current()
    }

    /// メッセージを追加して保存。
    pub fn append_message(
        &self,
        role: MessageRole,
        content: impl Into<String>,
    ) -> HistoryResult<()> {
        self.conversation_guard_mut()?
            .add_message(Message::new(role, content));
        self.save_current()
    }

    /// 条件に一致する末尾メッセージを削除。
    pub fn pop_last_if<F>(&self, predicate: F) -> HistoryResult<bool>
    where
        F: Fn(&Message) -> bool,
    {
        let removed = {
            let mut conv = self.conversation_guard_mut()?;
            if conv.messages.last().map(|m| predicate(m)).unwrap_or(false) {
                conv.messages.pop();
                true
            } else {
                false
            }
        };

        if removed {
            self.save_current()?;
        }

        Ok(removed)
    }

    fn conversation_guard(&self) -> HistoryResult<MutexGuard<'_, Conversation>> {
        self.conversation
            .lock()
            .map_err(|_| HistoryError::InvalidData("Conversation lock poisoned".into()))
    }

    fn conversation_guard_mut(&self) -> HistoryResult<MutexGuard<'_, Conversation>> {
        self.conversation
            .lock()
            .map_err(|_| HistoryError::InvalidData("Conversation lock poisoned".into()))
    }

    fn manager_guard(&self) -> HistoryResult<MutexGuard<'_, ConversationManager>> {
        self.manager
            .lock()
            .map_err(|_| HistoryError::InvalidData("Conversation manager lock poisoned".into()))
    }
}
