//! スクロール関連のユーティリティ

use gpui::*;

/// スクロールハンドルのラッパー
pub struct ScrollManager {
    handle: ScrollHandle,
    should_scroll_to_bottom: bool,
}

impl ScrollManager {
    /// 新しいスクロールマネージャーを作成
    pub fn new() -> Self {
        Self {
            handle: ScrollHandle::new(),
            should_scroll_to_bottom: false,
        }
    }

    /// スクロールハンドルへの参照を取得
    pub fn handle(&self) -> &ScrollHandle {
        &self.handle
    }

    /// 次のフレームで最下部にスクロールするようマーク
    pub fn mark_scroll_to_bottom(&mut self) {
        self.should_scroll_to_bottom = true;
    }

    /// スクロールが必要かチェックし、必要なら実行
    pub fn update(&mut self) {
        if self.should_scroll_to_bottom {
            self.handle.scroll_to_bottom();
            self.should_scroll_to_bottom = false;
        }
    }
}

impl Default for ScrollManager {
    fn default() -> Self {
        Self::new()
    }
}
