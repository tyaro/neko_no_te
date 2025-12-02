//! テキスト入力とIME対応のユーティリティ

use std::ops::Range;

/// テキスト入力の状態を管理する構造体
#[derive(Clone)]
pub struct TextInputState {
    /// 入力テキスト
    pub text: String,
    /// 選択範囲
    pub selected_range: Range<usize>,
    /// IME変換中のテキスト範囲
    pub marked_range: Option<Range<usize>>,
}

impl Default for TextInputState {
    fn default() -> Self {
        Self {
            text: String::new(),
            selected_range: 0..0,
            marked_range: None,
        }
    }
}

impl TextInputState {
    /// 新しいテキスト入力状態を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// テキストを取得
    pub fn text(&self) -> &str {
        &self.text
    }

    /// テキストをクリア
    pub fn clear(&mut self) {
        self.text.clear();
        self.selected_range = 0..0;
        self.marked_range = None;
    }

    /// 指定範囲のテキストを取得
    pub fn text_for_range(&self, range: Range<usize>) -> Option<String> {
        let chars: Vec<char> = self.text.chars().collect();
        if range.start <= chars.len() && range.end <= chars.len() {
            Some(chars[range.start..range.end].iter().collect())
        } else {
            None
        }
    }

    /// テキストを置換
    pub fn replace_text_in_range(&mut self, range: Option<Range<usize>>, new_text: &str) {
        let chars: Vec<char> = self.text.chars().collect();
        let replace_range = range.unwrap_or_else(|| self.selected_range.clone());

        let before: String = chars[..replace_range.start].iter().collect();
        let after: String = chars[replace_range.end..].iter().collect();
        self.text = format!("{}{}{}", before, new_text, after);

        let new_cursor = replace_range.start + new_text.chars().count();
        self.selected_range = new_cursor..new_cursor;
        self.marked_range = None;
    }

    /// テキストを置換してマーク（IME用）
    pub fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
    ) {
        let chars: Vec<char> = self.text.chars().collect();
        let replace_range = range.unwrap_or_else(|| {
            self.marked_range
                .clone()
                .unwrap_or_else(|| self.selected_range.clone())
        });

        let before: String = chars[..replace_range.start].iter().collect();
        let after: String = chars[replace_range.end..].iter().collect();
        self.text = format!("{}{}{}", before, new_text, after);

        let new_len = new_text.chars().count();
        let marked_start = replace_range.start;
        let marked_end = marked_start + new_len;

        self.marked_range = Some(marked_start..marked_end);

        if let Some(sel_range) = new_selected_range {
            self.selected_range = (marked_start + sel_range.start)..(marked_start + sel_range.end);
        } else {
            self.selected_range = marked_end..marked_end;
        }
    }

    /// マークを解除（IME確定）
    pub fn unmark_text(&mut self) {
        self.marked_range = None;
    }
}

/// EntityInputHandlerの実装をサポートするヘルパートレイト
pub trait TextInputHandler {
    /// TextInputStateへの参照を取得
    fn text_input_state(&self) -> &TextInputState;

    /// TextInputStateへの可変参照を取得
    fn text_input_state_mut(&mut self) -> &mut TextInputState;
}

/// EntityInputHandlerの標準実装を提供するマクロ
#[macro_export]
macro_rules! impl_entity_input_handler {
    ($type:ty) => {
        impl gpui::EntityInputHandler for $type {
            fn selected_text_range(
                &mut self,
                _ignore_disabled_input: bool,
                _window: &mut gpui::Window,
                _cx: &mut gpui::Context<Self>,
            ) -> Option<gpui::UTF16Selection> {
                let state = $crate::TextInputHandler::text_input_state(self);
                Some(gpui::UTF16Selection {
                    range: state.selected_range.clone(),
                    reversed: false,
                })
            }

            fn marked_text_range(
                &self,
                _window: &mut gpui::Window,
                _cx: &mut gpui::Context<Self>,
            ) -> Option<std::ops::Range<usize>> {
                $crate::TextInputHandler::text_input_state(self)
                    .marked_range
                    .clone()
            }

            fn text_for_range(
                &mut self,
                range_utf16: std::ops::Range<usize>,
                _adjusted_range: &mut Option<std::ops::Range<usize>>,
                _window: &mut gpui::Window,
                _cx: &mut gpui::Context<Self>,
            ) -> Option<String> {
                $crate::TextInputHandler::text_input_state(self).text_for_range(range_utf16)
            }

            fn replace_text_in_range(
                &mut self,
                replacement_range: Option<std::ops::Range<usize>>,
                text: &str,
                _window: &mut gpui::Window,
                cx: &mut gpui::Context<Self>,
            ) {
                $crate::TextInputHandler::text_input_state_mut(self)
                    .replace_text_in_range(replacement_range, text);
                cx.notify();
            }

            fn replace_and_mark_text_in_range(
                &mut self,
                range_utf16: Option<std::ops::Range<usize>>,
                new_text: &str,
                new_selected_range: Option<std::ops::Range<usize>>,
                _window: &mut gpui::Window,
                cx: &mut gpui::Context<Self>,
            ) {
                $crate::TextInputHandler::text_input_state_mut(self)
                    .replace_and_mark_text_in_range(range_utf16, new_text, new_selected_range);
                cx.notify();
            }

            fn unmark_text(&mut self, _window: &mut gpui::Window, cx: &mut gpui::Context<Self>) {
                $crate::TextInputHandler::text_input_state_mut(self).unmark_text();
                cx.notify();
            }

            fn bounds_for_range(
                &mut self,
                _range_utf16: std::ops::Range<usize>,
                _bounds: gpui::Bounds<gpui::Pixels>,
                _window: &mut gpui::Window,
                _cx: &mut gpui::Context<Self>,
            ) -> Option<gpui::Bounds<gpui::Pixels>> {
                // 簡略化: 入力エリアの位置を返す
                None
            }

            fn character_index_for_point(
                &mut self,
                _point: gpui::Point<gpui::Pixels>,
                _window: &mut gpui::Window,
                _cx: &mut gpui::Context<Self>,
            ) -> Option<usize> {
                None
            }
        }
    };
}
