//! UI共通ユーティリティ
//!
//! このクレートは複数のビューで共有されるUI機能を提供します：
//! - テキスト入力とIME対応
//! - スクロール機能
//! - 共通のUIコンポーネント

pub mod text_input;
pub mod scroll_utils;

pub use text_input::*;
pub use scroll_utils::*;
