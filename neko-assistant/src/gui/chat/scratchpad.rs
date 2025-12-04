use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::*;
use gpui::{Context, Entity, Window};
use gpui_component::button::Button;
use gpui_component::input::InputState;
use gpui_component::WindowExt;
use neko_ui::ConsoleLogEntry;

use super::console_window::console_window;
use super::scratchpad_window::scratchpad_window;

#[derive(Clone)]
pub struct ScratchpadManager {
    file_path: PathBuf,
    editor_input: Entity<InputState>,
}

impl ScratchpadManager {
    pub fn new(repo_root: &Path, editor_input: Entity<InputState>) -> Self {
        let file_path = repo_root.join("scratchpad.md");
        Self {
            file_path,
            editor_input,
        }
    }

    pub fn load(
        &self,
        window: &mut Window,
        cx: &mut Context<super::ChatView>,
    ) -> Result<(), String> {
        match fs::read_to_string(&self.file_path) {
            Ok(content) => {
                let _ = self.editor_input.update(cx, |state, cx| {
                    state.set_value(&content, window, cx);
                });
                cx.notify();
                Ok(())
            }
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    return Ok(());
                }
                Err(format!(
                    "Failed to load scratchpad {}: {}",
                    self.file_path.display(),
                    err
                ))
            }
        }
    }

    pub fn save(&self, cx: &mut Context<super::ChatView>) -> Result<(), String> {
        let contents = self.editor_input.read(cx).value();
        fs::write(&self.file_path, contents.as_str()).map_err(|err| {
            format!(
                "Failed to save scratchpad {}: {}",
                self.file_path.display(),
                err
            )
        })
    }

    pub fn open_sheet(
        &self,
        view: gpui::Entity<super::ChatView>,
        console_logs: Vec<ConsoleLogEntry>,
        window: &mut Window,
        cx: &mut Context<super::ChatView>,
    ) {
        if let Err(err) = self.load(window, cx) {
            eprintln!("{}", err);
        }

        let editor_input = self.editor_input.clone();
        let view_for_load = view.clone();
        let view_for_save = view.clone();
        let manager_for_load = Arc::new(self.clone());
        let manager_for_save = Arc::new(self.clone());

        let _ = window.open_sheet(cx, move |sheet, window, _app| {
            let load_manager = manager_for_load.clone();
            let load_listener = window.listener_for(
                &view_for_load,
                move |_this: &mut super::ChatView, _event: &gpui::ClickEvent, window, cx| {
                    if let Err(err) = load_manager.load(window, cx) {
                        eprintln!("{}", err);
                    }
                },
            );
            let save_manager = manager_for_save.clone();
            let save_listener = window.listener_for(
                &view_for_save,
                move |_this: &mut super::ChatView, _event: &gpui::ClickEvent, _window, cx| {
                    if let Err(err) = save_manager.save(cx) {
                        eprintln!("{}", err);
                    }
                },
            );

            let load_btn = Button::new("sheet_scratchpad_reload")
                .label("Reload")
                .on_click(load_listener);
            let save_btn = Button::new("sheet_scratchpad_save")
                .label("Save")
                .on_click(save_listener);

            sheet
                .title(
                    div()
                        .text_sm()
                        .text_color(rgb(0xffffff))
                        .child("Scratchpad"),
                )
                .size(px(400.0))
                .child(scratchpad_window(&editor_input, load_btn, save_btn))
                .child(div().pt_2().child(console_window(&console_logs)))
        });
    }
}
