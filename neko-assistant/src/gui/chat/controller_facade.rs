use std::sync::Arc;

use chat_core::{ChatCommand, ChatController, ChatState, ControllerError, ControllerSubscription};

/// ChatController とのやり取りをカプセル化する薄いラッパー。
pub struct ChatControllerFacade {
    controller: Arc<ChatController>,
    _subscription: ControllerSubscription,
}

impl ChatControllerFacade {
    pub fn new(controller: Arc<ChatController>, subscription: ControllerSubscription) -> Self {
        Self {
            controller,
            _subscription: subscription,
        }
    }

    pub fn controller(&self) -> Arc<ChatController> {
        Arc::clone(&self.controller)
    }

    pub fn handle_command(&self, command: ChatCommand) -> Result<(), ControllerError> {
        self.controller.handle_command(command)
    }

    pub fn state_snapshot(&self) -> ChatState {
        self.controller.state_snapshot()
    }
}
