use gpui::*;
use gpui_component::input::{Input, InputState};
use gpui_component::select::{Select, SelectItem, SelectState};
use gpui_component::StyledExt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelPreset {
    pub id: String,
    pub label: String,
}

impl ModelPreset {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

impl SelectItem for ModelPreset {
    type Value = String;

    fn title(&self) -> SharedString {
        SharedString::from(self.label.clone())
    }

    fn value(&self) -> &Self::Value {
        &self.id
    }

    fn matches(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.label.to_lowercase().contains(&query) || self.id.to_lowercase().contains(&query)
    }
}

pub fn model_selector(
    select_state: &gpui::Entity<SelectState<Vec<ModelPreset>>>,
    input: &gpui::Entity<InputState>,
) -> impl IntoElement {
    let preset_select = Select::new(select_state)
        .placeholder("Select model...")
        .menu_width(px(320.0))
        .w(px(240.0))
        .cleanable(true);

    div()
        .h_flex()
        .gap_1()
        .items_center()
        .child(preset_select)
        .child(Input::new(input).w(px(220.0)).text_sm())
}
