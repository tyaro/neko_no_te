use gpui::Styled;
use gpui::{
    div, px, rgb, App, AppContext, Application, Context, IntoElement, ParentElement, Render,
    SharedString, TitlebarOptions, Window, WindowOptions,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::StyledExt;

struct ScrollablePreview;

impl ScrollablePreview {
    fn new() -> Self {
        Self
    }

    fn header_section(&self) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xf1f5ff))
                    .child("Scrollable experiments"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xb6c5ff))
                    .child("Theme-aware palettes"),
            )
    }

    fn vertical_log_section(&self) -> impl IntoElement {
        let entries = (1..=24)
            .map(|i| {
                div()
                    .w_full()
                    .paddings(px(6.0))
                    .bg(if i % 2 == 0 {
                        rgb(0x141516)
                    } else {
                        rgb(0x0c0d10)
                    })
                    .border_r(px(6.0))
                    .child(format!("Vertical log entry #{:02}", i))
            })
            .collect::<Vec<_>>();

        div()
            .bg(rgb(0x14131f))
            .border_r(px(10.0))
            .paddings(px(12.0))
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xecf4ff))
                    .child("Vertical overflow"),
            )
            .child(
                div()
                    .h(px(200.0))
                    .v_flex()
                    .overflow_y_scrollbar()
                    .gap_1()
                    .children(entries)
                    .paddings(px(6.0))
                    .bg(rgb(0x0d0d12)),
            )
    }

    fn horizontal_tracks_section(&self) -> impl IntoElement {
        let panels = (1..=18)
            .map(|i| {
                div()
                    .w(px(128.0))
                    .h(px(80.0))
                    .border_r(px(8.0))
                    .paddings(px(8.0))
                    .bg(if i % 2 == 0 {
                        rgb(0x1e1f2e)
                    } else {
                        rgb(0x1a1a28)
                    })
                    .child(format!("Track {i}"))
            })
            .collect::<Vec<_>>();

        div()
            .bg(rgb(0x141424))
            .border_r(px(10.0))
            .paddings(px(12.0))
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xdce0ff))
                    .child("Horizontal overflow"),
            )
            .child(
                div()
                    .w(px(400.0))
                    .h_flex()
                    .overflow_x_scrollbar()
                    .gap_2()
                    .children(panels)
                    .paddings(px(6.0))
                    .bg(rgb(0x0c0c15))
                    .h(px(110.0)),
            )
    }

    fn stepped_section(&self) -> impl IntoElement {
        let steps = (1..=20)
            .map(|i| {
                div()
                    .w_full()
                    .h(px(26.0 + (i as f32) * 3.0))
                    .paddings(px(6.0))
                    .bg(if i % 2 == 0 {
                        rgb(0x101015)
                    } else {
                        rgb(0x181817)
                    })
                    .child(format!("Stepped row {i}"))
            })
            .collect::<Vec<_>>();

        div()
            .bg(rgb(0x16161f))
            .border_r(px(10.0))
            .paddings(px(12.0))
            .gap_2()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xe2e4ff))
                    .child("Stepped heights"),
            )
            .child(
                div()
                    .h(px(180.0))
                    .v_flex()
                    .overflow_y_scrollbar()
                    .gap_1()
                    .children(steps)
                    .paddings(px(6.0))
                    .bg(rgb(0x0a0a0f)),
            )
    }

    fn auto_scroll_note(&self) -> impl IntoElement {
        div()
            .bg(rgb(0x0d0d12))
            .border_r(px(10.0))
            .paddings(px(10.0))
            .gap_1()
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xe4ebff))
                    .child("Auto-scroll idea"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0xc3d3ff))
                    .child("Use ScrollHandle + StatefulInteractiveElement::track_scroll to pin the view at the bottom when new elements are appended."),
            )
    }
}

impl Render for ScrollablePreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0x05060c))
            .paddings(px(18.0))
            .gap_2()
            .child(self.header_section())
            .child(
                div()
                    .v_flex()
                    .gap_2()
                    .child(self.vertical_log_section())
                    .child(self.horizontal_tracks_section())
                    .child(self.stepped_section())
                    .child(self.auto_scroll_note()),
            )
    }
}

fn main() -> std::io::Result<()> {
    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);
        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some(SharedString::from("Scrollable Demo")),
                ..Default::default()
            }),
            ..WindowOptions::default()
        };

        cx.open_window(options, move |_window, cx| {
            cx.new(|_| ScrollablePreview::new())
        })
        .unwrap();
        cx.activate(true);
    });
    Ok(())
}
