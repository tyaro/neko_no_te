use crate::mcp_client::{load_mcp_config, save_mcp_config, McpServerConfig};
use gpui::*;
use gpui_component::button::Button;
use gpui_component::input::{Input, InputState};
use gpui_component::{Root, StyledExt};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct McpManagerView {
    configs: Rc<RefCell<Vec<McpServerConfig>>>,
    selected: Rc<RefCell<Option<usize>>>,
    name_input: Entity<InputState>,
    command_input: Entity<InputState>,
    args_input: Entity<InputState>,
    env_input: Entity<InputState>,
    status: Rc<RefCell<Option<String>>>,
}

impl McpManagerView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let configs = load_mcp_config().unwrap_or_else(|e| {
            eprintln!("Failed to load MCP config: {}", e);
            Vec::new()
        });

        let name_input = cx.new(|cx| InputState::new(window, cx));
        let command_input = cx.new(|cx| InputState::new(window, cx));
        let args_input = cx.new(|cx| InputState::new(window, cx));
        let env_input = cx.new(|cx| InputState::new(window, cx));

        Self {
            configs: Rc::new(RefCell::new(configs)),
            selected: Rc::new(RefCell::new(None)),
            name_input,
            command_input,
            args_input,
            env_input,
            status: Rc::new(RefCell::new(None)),
        }
    }

    fn set_form(&mut self, window: &mut Window, cx: &mut Context<Self>, cfg: &McpServerConfig) {
        let _ = self
            .name_input
            .update(cx, |state, cx| state.set_value(&cfg.name, window, cx));
        let _ = self
            .command_input
            .update(cx, |state, cx| state.set_value(&cfg.command, window, cx));
        let args_value = cfg.args.join(", ");
        let _ = self
            .args_input
            .update(cx, |state, cx| state.set_value(&args_value, window, cx));
        let env_text = cfg
            .env
            .as_ref()
            .map(|env| {
                env.iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();
        let _ = self
            .env_input
            .update(cx, |state, cx| state.set_value(&env_text, window, cx));
    }

    fn clear_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for input in [
            &self.name_input,
            &self.command_input,
            &self.args_input,
            &self.env_input,
        ] {
            let _ = input.update(cx, |state, cx| state.set_value("", window, cx));
        }
        *self.selected.borrow_mut() = None;
    }

    fn set_status(&mut self, msg: Option<String>) {
        *self.status.borrow_mut() = msg;
    }

    fn parse_form(&self, cx: &Context<Self>) -> Result<McpServerConfig, String> {
        let name = self.name_input.read(cx).value().trim().to_string();
        if name.is_empty() {
            return Err("Server name is required".into());
        }

        let command = self.command_input.read(cx).value().trim().to_string();
        if command.is_empty() {
            return Err("Command is required".into());
        }

        let args_field = self.args_input.read(cx).value();
        let args = args_field
            .split(|c| c == '\n' || c == ',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let env_text = self.env_input.read(cx).value();
        let env = parse_env(&env_text)?;

        Ok(McpServerConfig {
            name,
            command,
            args,
            env,
        })
    }

    fn edit_entry(&mut self, idx: usize, window: &mut Window, cx: &mut Context<Self>) {
        let cfg = self.configs.borrow().get(idx).cloned();
        if let Some(cfg) = cfg {
            self.set_form(window, cx, &cfg);
            *self.selected.borrow_mut() = Some(idx);
            self.set_status(Some(format!("Editing '{}'.", cfg.name)));
        }
    }

    fn remove_entry(&mut self, idx: usize) {
        let removed = {
            let mut list = self.configs.borrow_mut();
            if idx >= list.len() {
                return;
            }
            list.remove(idx)
        };
        *self.selected.borrow_mut() = None;
        let result = {
            let list = self.configs.borrow();
            save_mcp_config(&list)
        };
        match result {
            Ok(_) => self.set_status(Some(format!("Removed '{}'.", removed.name))),
            Err(e) => self.set_status(Some(format!("Failed to save config: {}", e))),
        }
    }

    fn save_entry(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.parse_form(cx) {
            Ok(cfg) => {
                // First, get the selected index without holding any borrows
                let selected_idx = *self.selected.borrow();

                // Update the configs list
                {
                    let mut list = self.configs.borrow_mut();
                    if let Some(idx) = selected_idx {
                        if idx < list.len() {
                            list[idx] = cfg.clone();
                        } else {
                            list.push(cfg.clone());
                        }
                    } else {
                        list.push(cfg.clone());
                    }
                }

                // Update selected index separately
                if selected_idx.is_none() {
                    let list_len = self.configs.borrow().len();
                    *self.selected.borrow_mut() = Some(list_len - 1);
                }

                let save_result = {
                    let list = self.configs.borrow();
                    save_mcp_config(&list)
                };

                match save_result {
                    Ok(_) => {
                        self.set_status(Some(format!("Saved '{}'.", cfg.name)));
                        self.set_form(window, cx, &cfg);
                    }
                    Err(e) => self.set_status(Some(format!("Failed to save: {}", e))),
                }
            }
            Err(err) => self.set_status(Some(err)),
        }
    }
}

impl Render for McpManagerView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        gpui_component::init(cx);

        let mut root = div().v_flex().gap_4().p_4().size_full();
        root = root.child(div().child("MCP Server Manager").text_size(px(22.0)));

        let servers = {
            let list = self.configs.borrow();
            if list.is_empty() {
                div().child("No MCP servers configured yet.")
            } else {
                let mut col = div().v_flex().gap_2();
                for (idx, cfg) in list.iter().enumerate() {
                    let name = cfg.name.clone();
                    let edit_listener = cx.listener({
                        let idx = idx;
                        move |this: &mut Self,
                              _ev: &ClickEvent,
                              window: &mut Window,
                              cx: &mut Context<Self>| {
                            this.edit_entry(idx, window, cx);
                        }
                    });
                    let remove_listener = cx.listener({
                        let idx = idx;
                        move |this: &mut Self,
                              _ev: &ClickEvent,
                              _window: &mut Window,
                              _cx: &mut Context<Self>| {
                            this.remove_entry(idx);
                        }
                    });
                    col = col.child(
                        div()
                            .h_flex()
                            .gap_2()
                            .items_center()
                            .child(div().flex_1().child(format!(
                                "{} → {}",
                                name,
                                cfg.command.clone()
                            )))
                            .child(
                                Button::new(SharedString::from(format!("edit_{}", idx)))
                                    .label("Edit")
                                    .on_click(edit_listener),
                            )
                            .child(
                                Button::new(SharedString::from(format!("remove_{}", idx)))
                                    .label("Remove")
                                    .on_click(remove_listener),
                            ),
                    );
                }
                col
            }
        };

        root = root.child(servers);

        root = root.child(
            div()
                .v_flex()
                .gap_3()
                .child(div().child("Server Name"))
                .child(Input::new(&self.name_input))
                .child(div().child("Command"))
                .child(Input::new(&self.command_input))
                .child(div().child("Arguments (comma or newline separated)"))
                .child(Input::new(&self.args_input))
                .child(div().child("Env (key=value per line)"))
                .child(Input::new(&self.env_input)),
        );

        let actions = div()
            .h_flex()
            .gap_2()
            .child(
                Button::new("save_entry")
                    .label("Save Entry")
                    .on_click(cx.listener(|this, _ev, window, cx| {
                        this.save_entry(window, cx);
                    })),
            )
            .child(
                Button::new("clear_form")
                    .label("Clear Form")
                    .on_click(cx.listener(|this, _ev, window, cx| {
                        this.clear_form(window, cx);
                    })),
            );
        root = root.child(actions);

        if let Some(msg) = self.status.borrow().as_ref() {
            root = root.child(div().text_color(rgb(0xcccccc)).child(msg.clone()));
        }

        root
    }
}

pub fn open_mcp_manager_window(cx: &mut App) {
    let _ = cx.open_window(WindowOptions::default(), move |window, cx| {
        let view = cx.new(|cx| McpManagerView::new(window, cx));
        cx.new(|cx| Root::new(view, window, cx))
    });
}

fn parse_env(text: &str) -> Result<Option<HashMap<String, String>>, String> {
    let mut map = HashMap::new();
    for (line_no, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            map.insert(key.trim().to_string(), value.trim().to_string());
        } else {
            return Err(format!(
                "Invalid env entry on line {} (expected key=value)",
                line_no + 1
            ));
        }
    }
    if map.is_empty() {
        Ok(None)
    } else {
        Ok(Some(map))
    }
}
