use chat_core::{
    ChatCommand, ChatController, ChatControllerConfig, ChatEvent, ChatState,
    ControllerSubscription, ConversationService,
};
use chat_history::{Conversation, ConversationManager, Message, MessageRole};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tempfile::{tempdir, TempDir};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio::time::timeout;

fn conversation_service_with_temp_storage(temp_dir: &TempDir) -> ConversationService {
    let manager = ConversationManager::new(temp_dir.path()).expect("failed to init manager");
    let manager = Arc::new(Mutex::new(manager));

    let mut conversation = Conversation::new("Test Conversation");
    conversation.add_message(Message::new(
        MessageRole::System,
        "Welcome to Neko Assistant".to_string(),
    ));
    let conversation = Arc::new(Mutex::new(conversation));

    ConversationService::new(conversation, manager)
}

#[test]
fn conversation_service_appends_and_persists_messages() {
    let temp_dir = tempdir().unwrap();
    let service = conversation_service_with_temp_storage(&temp_dir);

    service
        .append_message(MessageRole::User, "Hello from user")
        .unwrap();
    service
        .append_message(MessageRole::Assistant, "Responder")
        .unwrap();

    let conversation_id = service.current_conversation_id().unwrap();
    let rehydrated_manager = ConversationManager::new(temp_dir.path()).unwrap();
    let loaded = rehydrated_manager.load(&conversation_id).unwrap();

    assert_eq!(loaded.messages.len(), 3); // includes welcome message
    assert_eq!(loaded.messages.last().unwrap().content, "Responder");
}

#[test]
fn conversation_service_pop_last_if_removes_matching_message() {
    let temp_dir = tempdir().unwrap();
    let service = conversation_service_with_temp_storage(&temp_dir);

    service
        .append_message(MessageRole::User, "Keep this")
        .unwrap();
    service
        .append_message(MessageRole::User, "Remove this")
        .unwrap();

    let removed = service
        .pop_last_if(|msg| msg.content == "Remove this")
        .unwrap();
    assert!(removed);

    let messages = service.current_messages();
    assert_eq!(messages.last().unwrap().content, "Keep this");
}

struct ControllerHarness {
    controller: ChatController,
    events_rx: UnboundedReceiver<ChatEvent>,
    _subscription: ControllerSubscription,
    _temp_dir: TempDir,
}

impl ControllerHarness {
    fn new() -> Self {
        let temp_dir = tempdir().unwrap();
        let service = conversation_service_with_temp_storage(&temp_dir);
        let controller = ChatController::new(ChatControllerConfig {
            conversation_service: service,
            active_model: "phi4-mini:3.8b".to_string(),
            use_langchain: false,
            ollama_url: "http://localhost:11434".to_string(),
            mcp_manager: None,
            mcp_configs: Vec::new(),
            prompt_registry: None,
            welcome_message: "Welcome to Neko Assistant".to_string(),
        });

        let (tx, rx) = unbounded_channel();
        let subscription = controller.subscribe(move |event| {
            let _ = tx.send(event);
        });

        Self {
            controller,
            events_rx: rx,
            _subscription: subscription,
            _temp_dir: temp_dir,
        }
    }

    async fn next_event(&mut self) -> ChatEvent {
        timeout(Duration::from_secs(1), self.events_rx.recv())
            .await
            .expect("timed out waiting for ChatEvent")
            .expect("controller event channel closed")
    }

    async fn next_state(&mut self) -> ChatState {
        loop {
            match self.next_event().await {
                ChatEvent::StateChanged => return self.controller.state_snapshot(),
                _ => continue,
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn chat_controller_emits_state_change_on_user_message() {
    let mut harness = ControllerHarness::new();

    harness
        .controller
        .handle_command(ChatCommand::SendUserMessage("テスト".to_string()))
        .unwrap();

    let mut saw_assistant_reply = false;
    for _ in 0..3 {
        let state = harness.next_state().await;
        if state
            .messages
            .iter()
            .any(|msg| msg.role == MessageRole::Assistant && msg.content == "(echo) テスト")
        {
            saw_assistant_reply = true;
            break;
        }
    }

    assert!(
        saw_assistant_reply,
        "assistant echo response was not recorded"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn chat_controller_emits_model_changed_event() {
    let mut harness = ControllerHarness::new();
    let next_model = "qwen3:4b-instruct".to_string();

    harness
        .controller
        .handle_command(ChatCommand::SwitchModel(next_model.clone()))
        .unwrap();

    let mut observed_event = false;
    for _ in 0..3 {
        match harness.next_event().await {
            ChatEvent::ModelChanged => {
                observed_event = true;
                break;
            }
            _ => continue,
        }
    }

    assert!(observed_event, "model change event not observed");
    assert_eq!(harness.controller.state_snapshot().active_model, next_model);
}

#[tokio::test(flavor = "multi_thread")]
async fn chat_controller_create_conversation_resets_history() {
    let mut harness = ControllerHarness::new();

    harness
        .controller
        .handle_command(ChatCommand::CreateConversation)
        .unwrap();

    let state = harness.next_state().await;
    assert_eq!(state.messages.len(), 1);
    assert_eq!(state.messages[0].role, MessageRole::System);
    assert_eq!(state.messages[0].content, "Welcome to Neko Assistant");
}
