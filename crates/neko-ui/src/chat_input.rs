//! チャット入力コンポーネント
//!
//! gpui-componentのInputStateをラップした高性能な複数行入力コンポーネント。
//! IME入力、自動高さ調整、カスタムキーバインディングに対応。

use gpui::*;
use gpui_component::input::{InputEvent, InputState};

/// チャット入力のキー設定
#[derive(Clone, Debug, PartialEq)]
pub enum SendKeyConfig {
    /// Enterで送信、Shift+Enterで改行
    Enter,
    /// Ctrl+Enterで送信、Enterで改行
    CtrlEnter,
}

/// チャット入力コンポーネント
///
/// `gpui-component`の`InputState`をラップし、チャット用の機能を追加：
/// - 複数行入力（1〜5行の自動成長）
/// - IME完全対応
/// - カスタマイズ可能な送信キー
/// - プレースホルダー表示
pub struct ChatInput {
    input_state: Entity<InputState>,
    send_key: SendKeyConfig,
    _subscription: Subscription,
}

impl ChatInput {
    /// 新しいChatInputを作成
    ///
    /// # Arguments
    /// * `window` - GPUIウィンドウ
    /// * `cx` - コンテキスト
    /// * `placeholder` - プレースホルダーテキスト
    /// * `send_key` - 送信キー設定
    /// * `on_send` - メッセージ送信時のコールバック（テキストをクリアしてから呼ばれる）
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        placeholder: impl Into<SharedString>,
        send_key: SendKeyConfig,
        on_send: impl Fn(&str, &mut Window, &mut Context<Self>) + 'static,
    ) -> Self {
        // InputStateを作成（1〜5行の自動成長）
        let input_state = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(placeholder)
                .auto_grow(1, 5)
        });

        // InputEventを購読してEnterキーを処理
        let send_key_clone = send_key.clone();
        let input_clone = input_state.clone();
        let subscription = cx.subscribe_in(
            &input_state,
            window,
            move |_view, _state, event: &InputEvent, window, cx| {
                if let InputEvent::PressEnter { secondary } = event {
                    let should_send = match send_key_clone {
                        SendKeyConfig::Enter => !secondary, // Enterで送信、Shift+Enterで改行
                        SendKeyConfig::CtrlEnter => *secondary, // Ctrl+Enterで送信、Enterで改行
                    };

                    if should_send {
                        // テキストを取得
                        let text = input_clone.read(cx).text().to_string();
                        if !text.trim().is_empty() {
                            // コールバックを呼び出し（クリアは呼び出し側で行う）
                            on_send(&text, window, cx);
                        }
                    }
                }
            },
        );

        Self {
            input_state,
            send_key,
            _subscription: subscription,
        }
    }

    /// 入力テキストを取得
    pub fn text(&self, cx: &App) -> String {
        self.input_state.read(cx).text().to_string()
    }

    /// InputStateのEntityを取得（Inputコンポーネントのレンダリング用）
    pub fn input_state(&self) -> &Entity<InputState> {
        &self.input_state
    }

    /// 送信キー設定を取得
    pub fn send_key(&self) -> &SendKeyConfig {
        &self.send_key
    }
}
