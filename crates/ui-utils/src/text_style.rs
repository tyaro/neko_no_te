use gpui::{px, Styled};

/// Convenience helpers for text sizing missing from current gpui Component APIs.
pub trait TextStyleExt: Styled + Sized {
    fn text_md(self) -> Self {
        self.text_size(px(16.0))
    }
}

impl<T> TextStyleExt for T where T: Styled + Sized {}
