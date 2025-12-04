use gpui::*;
use gpui_component::StyledExt;

/// サイドバー + コンソール + メインペインを配置したレイアウト
pub fn chat_workspace(
    sidebar: impl IntoElement,
    console_panel: impl IntoElement,
    main_panel: impl IntoElement,
) -> Div {
    let content = div()
        .flex_1()
        .h_full()
        .h_flex()
        .child(console_panel)
        .child(main_panel);

    div()
        .h_flex()
        .w_full()
        .h_full()
        .child(sidebar)
        .child(content)
}
