use super::ChatView;
use chat_core::ChatEvent;
use gpui::{Context, Window};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// ChatController からのイベントをポーリングし、テストしやすいバッチ表現へ変換するループ。
#[derive(Clone)]
pub struct ChatEventLoop {
    rx: Arc<Mutex<mpsc::UnboundedReceiver<ChatEvent>>>,
}

/// `poll()` が返すイベント分類結果。テストではこの構造体を直接検証できる。
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ChatEventBatch {
    pub state_changed: bool,
    pub conversations_updated: bool,
    pub model_changed: bool,
    pub models_updated: bool,
    pub mcp_metadata_updated: bool,
    pub console_log_updated: bool,
    pub errors: Vec<String>,
}

impl ChatEventLoop {
    pub fn new(rx: Arc<Mutex<mpsc::UnboundedReceiver<ChatEvent>>>) -> Self {
        Self { rx }
    }

    /// 処理待ちイベントをまとめて取り出し、分類済みのバッチとして返す。
    pub fn poll(&self) -> ChatEventBatch {
        let mut batch = ChatEventBatch::default();
        if let Ok(mut rx) = self.rx.try_lock() {
            while let Ok(event) = rx.try_recv() {
                batch.record(event);
            }
        }
        batch
    }
}

impl ChatEventBatch {
    fn record(&mut self, event: ChatEvent) {
        match event {
            ChatEvent::StateChanged => self.state_changed = true,
            ChatEvent::ConversationsUpdated => self.conversations_updated = true,
            ChatEvent::ModelChanged => self.model_changed = true,
            ChatEvent::ModelsUpdated => self.models_updated = true,
            ChatEvent::McpMetadataUpdated => self.mcp_metadata_updated = true,
            ChatEvent::ConsoleLogUpdated => self.console_log_updated = true,
            ChatEvent::Error(message) => self.errors.push(message),
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.state_changed
            && !self.conversations_updated
            && !self.model_changed
            && !self.models_updated
            && !self.mcp_metadata_updated
            && !self.console_log_updated
            && self.errors.is_empty()
    }

    /// 収集済みバッチを ChatView に適用する。実行環境では従来通り UI を更新し、テストでは `dispatch_with()` を利用してロジックを検証できる。
    pub fn apply(self, view: &mut ChatView, window: &mut Window, cx: &mut Context<ChatView>) {
        if self.is_empty() {
            return;
        }

        let plan = self.into_dispatch_plan();

        if plan.mark_scroll_to_bottom {
            view.state.mark_scroll_to_bottom();
        }

        if plan.sync_active_model {
            let state = view.chat_state_snapshot();
            let active_model = state.active_model.clone();
            view.state
                .model_selector()
                .update_input_value(&active_model, window, cx);
            view.state
                .model_selector()
                .sync_selection(&state, window, cx);
        }

        if plan.sync_model_list {
            let state = view.chat_state_snapshot();
            view.state.model_selector().sync_items(&state, window, cx);
            view.state
                .model_selector()
                .sync_selection(&state, window, cx);
        }

        for message in plan.errors {
            eprintln!("Chat controller error: {}", message);
        }

        if plan.request_notify {
            cx.notify();
        }
    }

    fn into_dispatch_plan(self) -> DispatchPlan {
        let mut plan = DispatchPlan {
            errors: self.errors,
            ..Default::default()
        };

        if self.state_changed {
            plan.mark_scroll_to_bottom = true;
            plan.request_notify = true;
        }

        if self.conversations_updated {
            plan.request_notify = true;
        }

        if self.model_changed {
            plan.sync_active_model = true;
            plan.request_notify = true;
        }

        if self.models_updated {
            plan.sync_model_list = true;
            plan.request_notify = true;
        }

        if self.mcp_metadata_updated || self.console_log_updated {
            plan.request_notify = true;
        }

        plan
    }

    #[cfg(test)]
    pub fn dispatch_plan_for_test(self) -> DispatchPlan {
        self.into_dispatch_plan()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DispatchPlan {
    pub mark_scroll_to_bottom: bool,
    pub sync_active_model: bool,
    pub sync_model_list: bool,
    pub request_notify: bool,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll_collects_all_event_flags() {
        let (tx, rx) = mpsc::unbounded_channel();
        tx.send(ChatEvent::StateChanged).unwrap();
        tx.send(ChatEvent::ConversationsUpdated).unwrap();
        tx.send(ChatEvent::ModelChanged).unwrap();
        tx.send(ChatEvent::ModelsUpdated).unwrap();
        tx.send(ChatEvent::McpMetadataUpdated).unwrap();
        tx.send(ChatEvent::ConsoleLogUpdated).unwrap();
        tx.send(ChatEvent::Error("boom".into())).unwrap();

        let loop_ = ChatEventLoop::new(Arc::new(Mutex::new(rx)));
        let batch = loop_.poll();

        assert!(batch.state_changed);
        assert!(batch.conversations_updated);
        assert!(batch.model_changed);
        assert!(batch.models_updated);
        assert!(batch.mcp_metadata_updated);
        assert!(batch.console_log_updated);
        assert_eq!(batch.errors, vec!["boom".to_string()]);
        assert!(!batch.is_empty());
    }

    #[test]
    fn dispatch_plan_marks_expected_actions() {
        let batch = ChatEventBatch {
            state_changed: true,
            conversations_updated: true,
            model_changed: true,
            models_updated: true,
            mcp_metadata_updated: true,
            console_log_updated: false,
            errors: vec!["first".into(), "second".into()],
        };

        let plan = batch.dispatch_plan_for_test();

        assert!(plan.mark_scroll_to_bottom);
        assert!(plan.sync_active_model);
        assert!(plan.sync_model_list);
        assert!(plan.request_notify);
        assert_eq!(plan.errors, vec!["first", "second"]);
    }
}
