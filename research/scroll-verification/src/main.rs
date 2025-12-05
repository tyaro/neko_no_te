use gpui::Styled;
use gpui::{
    div, px, rgb, App, AppContext, Application, Context, IntoElement, ParentElement, Render,
    SharedString, TitlebarOptions, Window, WindowOptions,
};
use gpui_component::input::{Input, InputState};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{Root, StyledExt};

struct ScrollVerification {
    scratchpad_input: gpui::Entity<InputState>,
    console_logs: Vec<String>,
    chat_messages: Vec<String>,
}

impl ScrollVerification {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let scratchpad_input = cx.new(|cx| InputState::new(window, cx));
        
        // テストデータ生成
        let console_logs = (1..=50)
            .map(|i| format!("[Log {}] Console message", i))
            .collect();
        
        let chat_messages = (1..=30)
            .map(|i| format!("Chat message {}: Lorem ipsum dolor sit amet, consectetur adipiscing elit.", i))
            .collect();

        Self {
            scratchpad_input,
            console_logs,
            chat_messages,
        }
    }

    // スクラッチパッド + コンソールパネル（左側）
    fn scratchpad_console_panel(&self) -> impl IntoElement {
        let console_items: Vec<_> = self
            .console_logs
            .iter()
            .map(|log| {
                div()
                    .text_xs()
                    .text_color(rgb(0xcccccc))
                    .child(log.clone())
            })
            .collect();

        div()
            .h_full()
            .v_flex()
            // Scratchpad エリア
            .child(
                div()
                    .p_2()
                    .flex_shrink_0()
                    .border_b_1()
                    .border_color(rgb(0x333333))
                    .v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xaaaaaa))
                            .child("Scratchpad"),
                    )
                    .child(Input::new(&self.scratchpad_input).w_full().h(px(150.0)).text_sm()),
            )
            // Console エリア
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .v_flex()
                    .p_2()
                    .gap_1()
                    .child(
                        div()
                            .flex_shrink_0()
                            .text_sm()
                            .text_color(rgb(0xaaaaaa))
                            .child("Console"),
                    )
                    .child(
                        div().flex_1().min_h(px(0.0)).overflow_hidden().child(
                            div()
                                .size_full()
                                .v_flex()
                                .gap_1()
                                .overflow_y_scrollbar()
                                .children(console_items),
                        ),
                    ),
            )
    }

    // チャットパネル（右側）
    fn chat_panel(&self) -> impl IntoElement {
        let message_items: Vec<_> = self
            .chat_messages
            .iter()
            .map(|msg| {
                div()
                    .p_2()
                    .bg(rgb(0x1a1a1a))
                    .border_r(px(8.0))
                    .text_sm()
                    .text_color(rgb(0xdddddd))
                    .child(msg.clone())
            })
            .collect();

        div()
            .h_full()
            .v_flex()
            .gap_2()
            .p_3()
            .bg(rgb(0x0d0d0d))
            // ヘッダー
            .child(
                div()
                    .flex_shrink_0()
                    .text_lg()
                    .text_color(rgb(0xffffff))
                    .child("Chat"),
            )
            // メッセージエリア（スクロール可能）
            .child(
                div()
                    .flex_1()
                    .min_h(px(0.0))
                    .overflow_hidden()
                    .child(
                        div()
                            .size_full()
                            .v_flex()
                            .gap_2()
                            .overflow_y_scrollbar()
                            .children(message_items),
                    ),
            )
            // モデル選択
            .child(
                div()
                    .flex_shrink_0()
                    .p_2()
                    .bg(rgb(0x1a1a1a))
                    .text_sm()
                    .text_color(rgb(0xaaaaaa))
                    .child("Model: Phi-4 Mini"),
            )
            // 入力欄
            .child(
                div()
                    .flex_shrink_0()
                    .p_2()
                    .bg(rgb(0x1a1a1a))
                    .h(px(80.0))
                    .text_sm()
                    .text_color(rgb(0xaaaaaa))
                    .child("Input area"),
            )
    }
}

impl Render for ScrollVerification {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl IntoElement {
        gpui_component::init(cx);

        let console_panel = div()
            .w(px(340.0))
            .flex_shrink_0()
            .h_full()
            .bg(rgb(0x111111))
            .border_r_1()
            .border_color(rgb(0x242424))
            .child(self.scratchpad_console_panel());

        let main_panel = div()
            .flex_1()
            .h_full()
            .overflow_hidden()
            .child(self.chat_panel());

        let mut root_layout = div()
            .size_full()
            .bg(rgb(0x0a0a0a))
            .p_4()
            .child(
                div()
                    .size_full()
                    .h_flex()
                    .child(console_panel)
                    .child(main_panel),
            );

        if let Some(sheet_layer) = Root::render_sheet_layer(window, cx) {
            root_layout = root_layout.child(sheet_layer);
        }

        root_layout
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        title: Some(SharedString::from("Scroll Verification")),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| ScrollVerification::new(window, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .unwrap();
        })
        .detach();
    });
}
