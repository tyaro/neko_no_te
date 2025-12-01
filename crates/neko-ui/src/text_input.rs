//! IME対応のテキスト入力コンポーネント

use gpui::*;
use ui_utils::{TextInputState, TextInputHandler, impl_entity_input_handler};

/// テキスト入力用のEntity
pub struct TextInput {
    text_state: TextInputState,
    focus_handle: FocusHandle,
    placeholder: Option<SharedString>,
}

impl TextInput {
    /// 新しいTextInputを作成
    pub fn new(cx: &mut App) -> Self {
        Self {
            text_state: TextInputState::new(),
            focus_handle: cx.focus_handle(),
            placeholder: None,
        }
    }
    
    /// プレースホルダーテキストを設定
    pub fn set_placeholder(&mut self, text: impl Into<SharedString>) {
        self.placeholder = Some(text.into());
    }
    
    /// テキストを取得
    pub fn text(&self) -> &str {
        self.text_state.text()
    }
    
    /// テキストをクリア
    pub fn clear(&mut self) {
        self.text_state.clear();
    }
    
    /// フォーカスハンドルを取得
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    
    /// テキストを範囲指定で置換（改行入力用など）
    pub fn replace_text_in_range(&mut self, range: Option<std::ops::Range<usize>>, text: &str) {
        self.text_state.replace_text_in_range(range, text);
    }
}

impl TextInputHandler for TextInput {
    fn text_input_state(&self) -> &TextInputState {
        &self.text_state
    }
    
    fn text_input_state_mut(&mut self) -> &mut TextInputState {
        &mut self.text_state
    }
}

impl_entity_input_handler!(TextInput);

/// テキスト入力のビジュアル要素
pub struct TextInputElement {
    input: Entity<TextInput>,
    min_height: Pixels,
    max_height: Pixels,
}

impl TextInputElement {
    pub fn new(input: Entity<TextInput>) -> Self {
        Self {
            input,
            min_height: px(80.0),
            max_height: px(200.0),
        }
    }
    
    pub fn min_height(mut self, height: Pixels) -> Self {
        self.min_height = height;
        self
    }
    
    pub fn max_height(mut self, height: Pixels) -> Self {
        self.max_height = height;
        self
    }
}

impl IntoElement for TextInputElement {
    type Element = Self;
    
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextInputElement {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();
    
    fn id(&self) -> Option<ElementId> {
        Some(ElementId::Name("text-input".into()))
    }
    
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }
    
    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        // divでラップして、track_focusとon_mouse_downを追加
        let focus_handle = self.input.read(cx).focus_handle.clone();
        let input_clone = self.input.clone();
        
        let mut element = div()
            .id("text-input-wrapper")
            .min_h(self.min_height)
            .max_h(self.max_height)
            .w_full()
            .p_2()
            .border_1()
            .border_color(rgb(0x4b5563))
            .bg(rgb(0x1f2937))
            .text_color(rgb(0xffffff))
            .cursor(CursorStyle::IBeam)
            .track_focus(&focus_handle)
            .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                // フォーカスを設定
                window.focus(&input_clone.read(cx).focus_handle());
            })
            .child("(input area)") // プレースホルダー
            .into_any_element();
        
        let layout_id = element.request_layout(window, cx);
        (layout_id, element)
    }
    
    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) {
        request_layout.prepaint(window, cx);
    }
    
    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        // ★★★ これが重要！ window.handle_input()を呼んでInputHandlerを登録 ★★★
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        
        // 実際のレンダリング
        request_layout.paint(window, cx);
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
