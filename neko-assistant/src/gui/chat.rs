use gpui::*;
use gpui_component::button::*;
use gpui_component::Root;
use gpui_component::StyledExt;
use gpui_component::input::{Input, InputEvent, InputState};
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use crate::plugins::PluginEntry;

// Simple chat view for the main window. It keeps a small message list and an
// input box. Pressing Enter in the input will append the message to the list.
struct ChatView {
    repo_root: PathBuf,
    plugins: Vec<PluginEntry>,
    input_state: gpui::Entity<InputState>,
    messages: Rc<RefCell<Vec<gpui::SharedString>>>,
    // keep subscriptions alive while the view exists
    _subscriptions: Vec<gpui::Subscription>,
}

impl ChatView {
    fn new(window: &mut gpui::Window, cx: &mut gpui::Context<Self>, repo_root: PathBuf, plugins: Vec<PluginEntry>) -> Self {
        // messages buffer (shared)
        let messages_rc = Rc::new(RefCell::new(vec![gpui::SharedString::from("Welcome to Neko Assistant")]));

        // create an InputState entity and subscribe to its events (PressEnter etc.)
        let input_state = cx.new(|cx| InputState::new(window, cx));

        // keep subscription alive so callback can push messages when Enter is pressed
        let _my_entity = cx.entity_id();
        let subs = vec![cx.subscribe_in(
            &input_state,
            window,
            {
                let messages_sub = messages_rc.clone();
            move |_this, state, ev: &InputEvent, window, cx| match ev {
                    InputEvent::PressEnter { .. } => {
                        let val = state.read(cx).value();
                        let trimmed = val.trim();
                        if trimmed.is_empty() {
                            return;
                        }

                        let you_ss = gpui::SharedString::from(format!("You: {}", trimmed));
                        let ai_ss = gpui::SharedString::from(format!("AI: {}", trimmed));
                        {
                            let mut v = messages_sub.borrow_mut();
                            let len = v.len();
                            if len >= 2 {
                                if v[len - 2] == you_ss && v[len - 1] == ai_ss {
                                    return;
                                }
                            }
                            v.push(you_ss);
                            v.push(ai_ss);
                        }

                        // clear input text via public API
                        let _ = state.update(cx, |view, cx| view.set_value("", window, cx));

                        cx.notify();
                    }
                    _ => {}
                }
            },
        )];

        Self {
            repo_root,
            plugins,
            input_state,
            messages: messages_rc,
            _subscriptions: subs,
        }
    }
}

impl gpui::Render for ChatView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        gpui_component::init(cx);

        // Toolbar with Plugins button
        let repo_clone = self.repo_root.clone();
        let plugins_clone = self.plugins.clone();
        let toolbar = div().h_flex().gap_2().child(
            Button::new(gpui::SharedString::from("open_plugins")).label(gpui::SharedString::from("Plugins")).on_click(move |_, _win, app_cx| {
                // open plugin manager window
                let repo_clone = repo_clone.clone();
                let plugins_clone = plugins_clone.clone();
                let _ = app_cx.open_window(WindowOptions::default(), move |window, cx| {
                    let view = cx.new(|_| PluginListView::new(&repo_clone, plugins_clone.clone()));
                    cx.new(|cx| Root::new(view, window, cx))
                });
            }),
        );

        // Messages column: show only the last N messages to avoid pushing input off-screen
        let tail = 100usize;
        let msgs_vec = self.messages.borrow();
        let start = if msgs_vec.len() > tail { msgs_vec.len() - tail } else { 0 };
        let mut msgs = div().v_flex().gap_2().size_full();
        for m in &msgs_vec[start..] {
            msgs = msgs.child(div().child(m.clone()));
        }

        // Input row: real gpui_component::Input entity (Send button removed — Enter alone sends)
        let input_row = div().h_flex().gap_2().child(Input::new(&self.input_state));

        div().v_flex().gap_3().size_full().child(toolbar).child(msgs.flex_grow()).child(input_row)
    }
}

struct PluginListView {
    _repo_root: PathBuf,
    plugins: Vec<PluginEntry>,
    selected: Option<usize>,
}

impl PluginListView {
    fn new(repo_root: &Path, plugins: Vec<PluginEntry>) -> Self {
        let selected = if plugins.len() > 0 { Some(0) } else { None };
        Self { _repo_root: repo_root.to_path_buf(), plugins, selected }
    }
}

impl gpui::Render for PluginListView {
    fn render(&mut self, _window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        // Init gpui-component helpers (safe to call each frame)
        gpui_component::init(cx);

        // Left: list of plugins as buttons. Right: details for selected plugin.
        let list_col = {
            let mut col = div().v_flex().gap_2().size_full();
            for (_i, entry) in self.plugins.iter().enumerate() {
                let title = entry
                    .metadata
                    .as_ref()
                    .and_then(|m| m.name.clone())
                    .unwrap_or_else(|| entry.dir_name.clone());

                // Convert to SharedString so ElementId can be built from it
                let title_ss = gpui::SharedString::from(title.clone());

                // Render as simple buttons (no click wiring yet).
                let btn = Button::new(title_ss.clone()).label(title_ss.clone());
                col = col.child(btn);
            }
            col
        };

        let detail_col = if let Some(idx) = self.selected {
            if let Some(entry) = self.plugins.get(idx) {
                let name = entry.metadata.as_ref().and_then(|m| m.name.clone()).unwrap_or_else(|| entry.dir_name.clone());
                let desc = entry.metadata.as_ref().and_then(|m| m.description.clone()).unwrap_or_default();
                let name_ss = gpui::SharedString::from(name);
                let desc_ss = gpui::SharedString::from(desc);
                div()
                    .v_flex()
                    .gap_2()
                    .child(div().child(name_ss))
                    .child(div().child(desc_ss))
                    .child(Button::new("enable").label("Enable"))
            } else {
                div().child("No plugin selected")
            }
        } else {
            div().child("No plugin selected")
        };

        // Root layout: horizontal split
        div()
            .h_flex()
            .gap_4()
            .size_full()
            .child(list_col.flex_grow())
            .child(detail_col.flex_grow())
    }
}

pub fn run_gui(repo_root: &Path) -> std::io::Result<()> {
    // Discover plugins
    let list = crate::plugins::discover_plugins(repo_root)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // take ownership of repo_root for async usage
    let repo_root_buf = repo_root.to_path_buf();

    Application::new().run(move |cx: &mut App| {
        // Initialize gpui-component once.
        gpui_component::init(cx);

        // clone owned handles for the async task
        let list_clone = list.clone();
        let repo_clone = repo_root_buf.clone();

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions::default(),
                move |window, cx| {
                    // Main window: chat view with toolbar that can open plugin manager
                    let view = cx.new(|cx| ChatView::new(window, cx, repo_clone.clone(), list_clone.clone()));
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .unwrap();
        })
        .detach();

        cx.activate(true);
    });

    Ok(())
}
