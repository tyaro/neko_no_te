# GPUI Scrollable Example

Small GPUI app that demonstrates `gpui_component::scroll::ScrollableElement` helpers, including `overflow_y_scrollbar()` and the `Scrollbar` overlay that ships with `gpui-component`. The layout mimics a terminal log so you can see how scrollbars appear when the list grows beyond the viewport.

## Run

```bash
cargo run -p gpui-scroll-example
```

Make sure the workspace dependencies are rebuilt after adding this crate (`cargo check -p gpui-scroll-example` was already run during development).

## Background

Implementation follows the official docs for `[gpui_component::scrollable](https://longbridge.github.io/gpui-component/docs/components/scrollable)`: wrap a flex column of `div` entries, call `.overflow_y_scrollbar()`, and embed the whole thing inside a styled container with `overflow_hidden()` so the scrollbar stays pinned to the visible area.

## What this demo shows

- a vertically scrollable log panel next to a horizontal track list and stepped-height stack so you can see scrollbars on multiple axes
- `ScrollableElement` wrappers that keep scrollbars pinned to their viewports and let you style the container independently
- a text note describing how to hook a `ScrollHandle` into a stateful chat view for auto-scrolling

## Auto-scroll chat (idea)

To keep a chat view pinned to its bottom when new messages arrive, store a `ScrollHandle` on your view state, then track it with the scrollable container you render:

```rust
pub struct ChatView {
	messages: Vec<String>,
	scroll_handle: ScrollHandle,
	should_auto_scroll: bool,
}

impl ChatView {
	fn add_message(&mut self, message: String) {
		self.messages.push(message);

		if self.should_auto_scroll {
			let max_offset = self.scroll_handle.max_offset();
			self.scroll_handle.set_offset(point(px(0.), max_offset.height));
		}
	}
}
```

`ScrollableElement` exposes `StatefulInteractiveElement::track_scroll`, so you can call it with that `ScrollHandle` to tie the scroll area and scrollbar together while you manipulate the handle manually.
