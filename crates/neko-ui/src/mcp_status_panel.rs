use gpui::*;
use gpui_component::accordion::Accordion;
use gpui_component::button::Button;
use gpui_component::StyledExt;

trait OverflowScrollExt: Sized {
    fn overflow_y_scroll(self) -> Self;
}

impl OverflowScrollExt for Div {
    fn overflow_y_scroll(mut self) -> Self {
        self.style().overflow.y = Some(Overflow::Scroll);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum McpServerStatusBadge {
    Unknown,
    Ready,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpServerItem {
    pub name: String,
    pub status: McpServerStatusBadge,
    pub tool_count: usize,
    pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpToolItem {
    pub server_name: String,
    pub tool_name: String,
    pub description: String,
}

pub fn mcp_status_panel(
    servers: &[McpServerItem],
    tools: &[McpToolItem],
    refresh_button: Button,
    manage_button: Button,
) -> Div {
    fn scroll_container(content: Div) -> Div {
        div()
            .max_h(px(200.0))
            .overflow_hidden()
            .child(div().size_full().overflow_y_scroll().child(content))
    }

    let header = div()
        .h_flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_sm()
                .text_color(rgb(0xffffff))
                .child("MCP Status"),
        )
        .child(
            div()
                .h_flex()
                .gap_2()
                .child(refresh_button)
                .child(manage_button),
        );

    let server_section = if servers.is_empty() {
        div()
            .text_sm()
            .text_color(rgb(0x999999))
            .child("No MCP servers configured.")
    } else {
        let rows = servers.iter().map(|item| {
            let status_badge = render_status_badge(&item.status, item.message.as_deref());
            div()
                .h_flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .text_sm()
                        .text_color(rgb(0xffffff))
                        .child(item.name.clone()),
                )
                .child(status_badge)
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xaaaaaa))
                        .child(format!("{} tools", item.tool_count)),
                )
        });
        div().v_flex().gap_1().children(rows)
    };

    let tools_section: AnyElement = if tools.is_empty() {
        div()
            .text_sm()
            .text_color(rgb(0x999999))
            .child("No tools reported yet.")
            .into_any_element()
    } else {
        let rows = tools.iter().map(|tool| {
            div()
                .v_flex()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xffffff))
                        .child(tool.tool_name.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xaaaaaa))
                        .child(format!("@{}", tool.server_name)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xcccccc))
                        .child(tool.description.clone()),
                )
        });
        scroll_container(div().v_flex().gap_2().children(rows)).into_any_element()
    };

    let accordion = Accordion::new("mcp_status_sections")
        .multiple(true)
        .bordered(false)
        .item(|item| {
            item.title(section_title("Servers", servers.len()))
                .open(true)
                .child(server_section)
        })
        .item(|item| {
            item.title(section_title("Tools", tools.len()))
                .open(!tools.is_empty())
                .child(tools_section)
        });

    div().p_2().gap_2().v_flex().child(header).child(accordion)
}

fn section_title(label: &str, count: usize) -> Div {
    let label_text = label.to_string();
    div()
        .h_flex()
        .items_center()
        .justify_between()
        .child(div().text_sm().text_color(rgb(0xffffff)).child(label_text))
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x888888))
                .child(format!("{}", count)),
        )
}

fn render_status_badge(status: &McpServerStatusBadge, message: Option<&str>) -> Div {
    let (label, bg_color, text_color) = match status {
        McpServerStatusBadge::Unknown => ("Unknown", rgb(0x444444), rgb(0xcccccc)),
        McpServerStatusBadge::Ready => ("Ready", rgb(0x1d8348), rgb(0xffffff)),
        McpServerStatusBadge::Error => ("Error", rgb(0x922b21), rgb(0xffffff)),
    };

    let mut row = div().h_flex().gap_1().items_center().child(
        div()
            .px(px(6.0))
            .py(px(2.0))
            .rounded_sm()
            .text_xs()
            .text_color(text_color)
            .bg(bg_color)
            .child(label),
    );

    if let (McpServerStatusBadge::Error, Some(msg)) = (status, message) {
        row = row.child(
            div()
                .text_xs()
                .text_color(rgb(0xf1948a))
                .child(msg.to_string()),
        );
    }

    row
}
