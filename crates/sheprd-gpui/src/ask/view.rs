// ABOUTME: Renders Ask Spaceport with a window-owned draft, focus and reading position.
// ABOUTME: Shows dated observation sources without borrowing or controlling a terminal pane.
use super::{Controller, controller, model::Interaction};
use crate::chat::{
    prepared::{Body, Part, Prepared},
    selection::Selection,
};
use crate::{
    config::{Config, Theme, corners},
    fonts::StyledFont,
    i18n::fl,
    multiline_input::MultilineInput,
};
use gpui::{prelude::*, *};
use sheprd_chat_protocol::{EntryKind, ToolContent};
use sheprd_control_protocol::protocol::observation::{self, Source};
use std::collections::HashSet;
use std::rc::Rc;
mod selection;
mod sources;
#[cfg(test)]
mod tests;

struct Row {
    id: u64,
    label: String,
    text: String,
    lines: Rc<[crate::release_notes::Line]>,
    sources: Vec<(Source, u64)>,
    folded: bool,
}

pub(crate) struct View {
    owner: Entity<Controller>,
    input: Entity<MultilineInput>,
    config: Config,
    theme: Theme,
    rows: Vec<Row>,
    prepared: Prepared,
    selection: Selection,
    geometry: selection::Geometry,
    drag: crate::chat::reader::Drag,
    drag_task: Option<Task<()>>,
    reader_focus: FocusHandle,
    expanded: HashSet<u64>,
    list: ListState,
    list_keys: Vec<(sheprd_chat_protocol::EntryId, u64)>,
    glide: crate::chat::glide::Glide,
    snapshots: u64,
    interaction: Interaction,
    _owner: Subscription,
    _input: Subscription,
}

impl View {
    fn text_ink(&self, color: u32) -> u32 {
        crate::contrast::ink_on_chrome(color, [self.theme.background; 3], 4.5)
    }

    #[cfg(feature = "integration-test")]
    pub(crate) fn fixture_draft(&self, cx: &App) -> String {
        self.input.read(cx).text().to_owned()
    }

    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    pub(crate) fn fixture_mark_composition(&self, window: &mut Window, cx: &mut App) -> bool {
        self.input.update(cx, |input, cx| {
            input.replace_and_mark_text_in_range(None, "観🚀", Some(1..3), window, cx);
            input
                .marked_text_range(window, cx)
                .is_some_and(|range| range.len() == 3)
        })
    }

    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    pub(crate) fn fixture_composition_bounds(
        &self,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<Bounds<Pixels>> {
        self.input.update(cx, |input, cx| {
            let marked = input.marked_text_range(window, cx)?;
            let selected = input.selected_text_range(false, window, cx)?;
            if selected.range != (marked.start + 1..marked.end) {
                return None;
            }
            input.bounds_for_range(marked, Bounds::default(), window, cx)
        })
    }

    #[cfg(all(feature = "integration-test", target_os = "macos"))]
    pub(crate) fn fixture_commit_composition(&self, window: &mut Window, cx: &mut App) -> bool {
        self.input.update(cx, |input, cx| {
            input.replace_text_in_range(None, "観測🛸", window, cx);
            input.marked_text_range(window, cx).is_none() && input.text().ends_with("観測🛸")
        })
    }

    pub(crate) fn new(config: Config, theme: Theme, cx: &mut Context<Self>) -> Self {
        let owner = controller(cx);
        let input = cx.new(|cx| {
            let mut input = MultilineInput::new(cx);
            input.set_appearance(config.ui.clone(), theme.clone(), cx);
            input.set_placeholder_with(|| fl!("ask-placeholder"), cx);
            input
        });
        let observed = cx.observe(&owner, |this, _, cx| {
            this.prepare(cx);
            if this.input.read(cx).text().is_empty()
                && let Some(text) = this.owner.read(cx).failed_draft.clone()
            {
                this.input
                    .update(cx, |input, cx| input.set_text_at(&text, text.len(), cx));
            }
            cx.notify();
        });
        let edited = cx.observe(&input, |_, _, cx| cx.notify());
        let list = ListState::new(0, ListAlignment::Top, px(200.));
        let glide = crate::chat::glide::Glide::new(&list);
        let mut view = Self {
            owner,
            input,
            config,
            theme,
            rows: Vec::new(),
            prepared: Prepared::default(),
            selection: Selection::default(),
            geometry: selection::Geometry::default(),
            drag: crate::chat::reader::Drag::default(),
            drag_task: None,
            reader_focus: cx.focus_handle(),
            expanded: HashSet::new(),
            list,
            list_keys: Vec::new(),
            glide,
            snapshots: 0,
            interaction: Interaction::default(),
            _owner: observed,
            _input: edited,
        };
        view.prepare(cx);
        view
    }

    pub(crate) fn focus(&self, window: &mut Window, cx: &mut App) {
        let focus = if self.owner.read(cx).configured {
            self.input.read(cx).focus_handle(cx)
        } else {
            self.reader_focus.clone()
        };
        window.focus(&focus, cx);
    }

    pub(crate) fn appearance(&mut self, config: &Config, theme: &Theme, cx: &mut Context<Self>) {
        if self.config.ui == config.ui
            && self.config.terminal == config.terminal
            && self.config.chat == config.chat
            && self.theme == *theme
        {
            return;
        }
        self.config = config.clone();
        self.theme = theme.clone();
        self.input.update(cx, |input, cx| {
            input.set_appearance(config.ui.clone(), theme.clone(), cx)
        });
        let top = self.list.logical_scroll_top();
        self.list.reset(self.rows.len());
        self.list.scroll_to(top);
        cx.notify();
    }

    fn prepare(&mut self, cx: &App) {
        let owner = self.owner.read(cx);
        let snapshot_changed = self.snapshots != owner.session.snapshots();
        if snapshot_changed {
            self.snapshots = owner.session.snapshots();
            self.prepared.clear();
            self.selection.clear();
            self.release_selection_drag();
        }
        let mut transcript = owner.session.transcript().clone();
        transcript.entries.retain(|entry| {
            matches!(
                entry.kind,
                EntryKind::User { .. }
                    | EntryKind::Agent { .. }
                    | EntryKind::Notice { .. }
                    | EntryKind::Tool(_)
            )
        });
        for entry in &mut transcript.entries {
            if let EntryKind::Agent { text } = &mut entry.kind {
                *text = sources::reading_text(text);
            }
        }
        let all_keys = owner.session.keys();
        let keys: Vec<_> = all_keys
            .into_iter()
            .filter(|(id, _)| transcript.entry(*id).is_some())
            .collect();
        let top = self.list.logical_scroll_top();
        if let Some(splices) = (!snapshot_changed)
            .then(|| crate::chat::list::splices(&self.list_keys, &keys))
            .flatten()
        {
            for splice in splices {
                crate::chat::glide::splice_in_place(&self.list, splice.range, splice.count);
            }
        } else {
            let reading = self
                .list_keys
                .get(top.item_ix)
                .and_then(|(id, _)| keys.iter().position(|(new, _)| new == id));
            self.list.reset(keys.len());
            if self.glide.stopped()
                && let Some(item_ix) = reading
            {
                self.list.scroll_to(ListOffset { item_ix, ..top });
            } else {
                self.glide.reset(&self.list);
            }
        }
        self.prepared.update_reading_lines(&transcript, &keys);
        self.list_keys = keys;
        self.selection.validate(self.prepared.text());
        let answer_sources = sources::answers(owner.session.transcript());
        self.rows = self
            .owner
            .read(cx)
            .session
            .transcript()
            .entries
            .iter()
            .filter_map(|entry| {
                let (label, text, sources, folded) = match &entry.kind {
                    EntryKind::User { text, .. } => {
                        (fl!("ask-you"), text.clone(), Vec::new(), false)
                    }
                    EntryKind::Agent { text } => (
                        "Pi".into(),
                        text.clone(),
                        answer_sources.get(&entry.id).cloned().unwrap_or_default(),
                        false,
                    ),
                    EntryKind::Notice { text, .. } => {
                        (fl!("ask-notice"), text.clone(), Vec::new(), false)
                    }
                    EntryKind::Tool(tool) => {
                        let sources = tool
                            .content
                            .iter()
                            .filter_map(|content| match content {
                                ToolContent::Text { text } => {
                                    serde_json::from_str::<observation::Reply>(text).ok()
                                }
                                _ => None,
                            })
                            .flat_map(|reply| match reply {
                                observation::Reply::Sessions {
                                    sessions,
                                    observed_at_ms,
                                    ..
                                } => sessions
                                    .into_iter()
                                    .map(|session| (session.source, observed_at_ms))
                                    .collect(),
                                observation::Reply::Excerpt {
                                    source,
                                    observed_at_ms,
                                    ..
                                } => vec![(*source, observed_at_ms)],
                                observation::Reply::Unavailable { .. } => Vec::new(),
                            })
                            .collect();
                        (
                            fl!("ask-observation"),
                            crate::notifications::safe_text(&tool.title, 300),
                            sources,
                            true,
                        )
                    }
                    _ => return None,
                };
                let lines = self
                    .prepared
                    .body(entry.id)
                    .and_then(|body| {
                        if let Body::Message(parts) = &*body
                            && let Some(Part::Text(lines)) = parts.first()
                        {
                            Some(lines.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| Rc::from([]));
                Some(Row {
                    id: entry.id.0,
                    label,
                    text,
                    lines,
                    sources,
                    folded,
                })
            })
            .collect();
    }

    fn submit(&mut self, cx: &mut Context<Self>) {
        let text = self.input.read(cx).text().to_owned();
        if self.owner.update(cx, |owner, cx| owner.submit(text, cx)) {
            self.input.update(cx, |input, cx| input.clear(cx));
        }
    }

    fn choose_pi(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.owner.read(cx).loading {
            return;
        }
        let handle = window.window_handle();
        let input = self.input.clone();
        let save = cx.background_executor().spawn(async {
            Config::save_preference(crate::config::preferences::Preference::AskAgent(Some(
                crate::config::chat::AskAgent::Pi,
            )))
        });
        cx.spawn(async move |this, cx| {
            let result = save.await;
            let selected = result.is_ok();
            let _ = this.update(cx, |this, cx| {
                this.owner.update(cx, |owner, cx| {
                    match result {
                        Ok(()) => {
                            owner.configured = true;
                        }
                        Err(error) => owner.problem = Some(error.to_string()),
                    }
                    cx.notify();
                });
                if this.owner.read(cx).configured {
                    this.config.chat.ask_agent = Some(crate::config::chat::AskAgent::Pi);
                }
            });
            if selected {
                let _ = handle.update(cx, |root, window, cx| {
                    if let Ok(view) = root.downcast::<crate::SheprdWindow>()
                        && view.read(cx).activity.panel == crate::sidebar::ActivityPanel::Ask
                    {
                        let focus = input.read(cx).focus_handle(cx);
                        window.focus(&focus, cx);
                    }
                });
            }
        })
        .detach();
        self.focus(window, cx);
    }

    fn button(&self, id: &'static str, label: String, enabled: bool) -> Stateful<Div> {
        let theme = &self.theme;
        let button = div()
            .id(id)
            .debug_selector(move || id.into())
            .role(Role::Button)
            .aria_label(label.clone())
            .px_3()
            .py_1()
            .rounded(px(corners::CONTROL))
            .text_size(px(self.config.ui.size))
            .border_1()
            .border_color(rgb(theme.active))
            .child(label)
            .when(enabled, |button| {
                button
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(theme.active)))
            })
            .when(!enabled, |button| button.opacity(0.45));
        #[cfg(all(feature = "integration-test", target_os = "macos"))]
        let button = crate::smoke::ask_provider::control(id.into(), button);
        button
    }

    fn row(&self, row: &Row, cx: &mut Context<Self>) -> Stateful<Div> {
        let copy = row.text.clone();
        let mut view = div()
            .id(("ask-entry", row.id))
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_2()
            .py_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(self.config.ui.size))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(self.text_ink(self.theme.muted)))
                            .child(row.label.clone()),
                    )
                    .child(
                        div()
                            .id(("ask-copy", row.id))
                            .role(Role::Button)
                            .aria_label(fl!("ask-copy"))
                            .cursor_pointer()
                            .text_size(px(self.config.ui.size))
                            .text_color(rgb(self.text_ink(self.theme.muted)))
                            .child(fl!("ask-copy"))
                            .on_click(move |_, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(copy.clone()))
                            }),
                    ),
            );
        if row.folded {
            let id = row.id;
            view = view.child(
                div()
                    .id(("ask-observation", id))
                    .role(Role::Button)
                    .cursor_pointer()
                    .text_size(px(self.config.ui.size))
                    .text_color(rgb(self.text_ink(self.theme.muted)))
                    .child(row.text.clone())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if !this.expanded.remove(&id) {
                            this.expanded.insert(id);
                        }
                        if let Some(index) = this.rows.iter().position(|row| row.id == id) {
                            crate::chat::glide::splice_in_place(&this.list, index..index + 1, 1);
                        }
                        cx.notify();
                    })),
            );
        } else {
            view = view.child(
                div().flex().flex_col().gap(px(2.)).min_w_0().children(
                    row.lines
                        .iter()
                        .enumerate()
                        .map(|(index, line)| self.reading_line(row.id, index, line, cx)),
                ),
            );
        }
        for (index, (source, observed)) in row.sources.iter().enumerate() {
            if row.folded && !self.expanded.contains(&row.id) {
                break;
            }
            let target = source.target.clone();
            let label = format!("{} · {} · {}", source.label, source.workspace, source.host);
            let at = i64::try_from(*observed)
                .ok()
                .and_then(chrono::DateTime::from_timestamp_millis)
                .map(|date| {
                    date.with_timezone(&chrono::Local)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                })
                .unwrap_or_else(|| "—".into());
            view = view.child(
                div()
                    .id((SharedString::from(format!("ask-source-{}", row.id)), index))
                    .debug_selector({
                        let id = row.id;
                        move || format!("ask-source-{id}-{index}")
                    })
                    .role(Role::Link)
                    .aria_label(label.clone())
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .text_size(px(self.config.ui.size))
                    .text_color(rgb(self.text_ink(self.theme.primary())))
                    .cursor_pointer()
                    .child(label)
                    .child(
                        div()
                            .text_color(rgb(self.text_ink(self.theme.muted)))
                            .child(fl!("ask-observed", time = at)),
                    )
                    .map(|button| {
                        #[cfg(all(feature = "integration-test", target_os = "macos"))]
                        let button = crate::smoke::ask_provider::control(
                            format!("ask-source-target-{}", source.target.0),
                            button,
                        );
                        button
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let target = target.clone();
                        let owner = this.owner.clone();
                        // The control handler updates a window after this event releases its borrow.
                        cx.defer(move |cx| {
                            let response = crate::control::open_observation_source(&target, cx);
                            if !matches!(response, sheprd_control_protocol::protocol::Response::Done { .. }) {
                                owner.update(cx, |owner, cx| {
                                    owner.problem = Some(match response {
                                        sheprd_control_protocol::protocol::Response::Error { code: sheprd_control_protocol::protocol::ErrorCode::Busy | sheprd_control_protocol::protocol::ErrorCode::WindowBusy, .. } => fl!("ask-source-busy"),
                                        _ => fl!("ask-source-unavailable"),
                                    });
                                    cx.notify();
                                });
                            }
                        });
                    })),
            );
        }
        view
    }
}

impl Render for View {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.geometry.clear();
        self.glide.tick(&self.list, crate::motion::now(), false);
        let owner = self.owner.read(cx);
        let (configured, loading, working, problem) = (
            owner.configured,
            owner.loading,
            owner.active,
            owner.problem.clone(),
        );
        let send =
            configured && !loading && !working && !self.input.read(cx).text().trim().is_empty();
        let header = div()
            .flex_none()
            .px_6()
            .py_4()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_3()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(self.config.ui.size + 3.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(fl!("ask-title")),
                    )
                    .child(
                        div()
                            .text_size(px(self.config.ui.size))
                            .text_color(rgb(self.text_ink(self.theme.muted)))
                            .child(fl!("ask-scope")),
                    ),
            )
            .when(configured, |header| {
                header.child(
                    div()
                        .text_color(rgb(self.text_ink(self.theme.muted)))
                        .child("Pi"),
                )
            })
            .child(
                self.button("ask-new", fl!("ask-new"), !loading)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if loading {
                            return;
                        }
                        if this.interaction.new_chat(this.owner.read(cx).active) {
                            let configured = this.config.chat.ask_agent.is_some();
                            this.owner
                                .update(cx, |owner, cx| owner.new_chat(configured, cx));
                        }
                        cx.notify();
                    })),
            );
        let mut transcript = div()
            .id("ask-transcript")
            .debug_selector(|| "ask-transcript".into())
            .flex_1()
            .min_h_0()
            .min_w_0()
            .px_6()
            .flex()
            .flex_col()
            .items_center();
        if !self.rows.is_empty() {
            transcript = transcript.child(
                list(
                    self.list.clone(),
                    cx.processor(|this, index: usize, _, cx| {
                        let Some(row) = this.rows.get(index) else {
                            return div().into_any_element();
                        };
                        div()
                            .w_full()
                            .min_w_0()
                            .flex()
                            .justify_center()
                            .child(
                                div()
                                    .w_full()
                                    .min_w_0()
                                    .when(this.config.chat.max_width != 0, |column| {
                                        column.max_w(px(f32::from(this.config.chat.max_width)))
                                    })
                                    .child(this.row(row, cx)),
                            )
                            .into_any_element()
                    }),
                )
                .flex_1()
                .size_full()
                .min_h_0(),
            );
        }
        if self.rows.is_empty() {
            transcript = transcript.child(
                div()
                    .w_full()
                    .max_w(px(640.))
                    .flex_1()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap_4()
                    .py_6()
                    .child(
                        div()
                            .text_size(px(self.config.ui.size + 5.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(fl!("ask-empty-title")),
                    )
                    .child(
                        div()
                            .text_color(rgb(self.text_ink(self.theme.muted)))
                            .child(fl!("ask-privacy")),
                    )
                    .when(!configured, |view| {
                        view.child(
                            self.button("ask-choose-pi", fl!("ask-choose-pi"), !loading)
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.choose_pi(window, cx)),
                                ),
                        )
                    })
                    .when(configured, |view| {
                        view.child(
                            self.button("ask-example", fl!("ask-example"), true)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    let text = fl!("ask-example");
                                    this.input.update(cx, |input, cx| {
                                        input.set_text_at(&text, text.len(), cx)
                                    });
                                    this.focus(window, cx);
                                })),
                        )
                    }),
            );
        }
        let composer = div()
            .flex_none()
            .w_full()
            .min_w_0()
            .when(self.config.chat.max_width != 0, |column| {
                column.max_w(px(f32::from(self.config.chat.max_width)))
            })
            .flex()
            .flex_col()
            .gap_2()
            .when(self.interaction.confirm_new, |composer| {
                composer.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .items_center()
                        .child(fl!("ask-new-confirm"))
                        .child(
                            self.button("ask-new-confirm", fl!("ask-stop-new"), true)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.interaction.confirm_new = false;
                                    let configured = this.config.chat.ask_agent.is_some();
                                    this.owner
                                        .update(cx, |owner, cx| owner.new_chat(configured, cx));
                                })),
                        )
                        .child(self.button("ask-new-keep", fl!("ask-keep"), true).on_click(
                            cx.listener(|this, _, _, cx| {
                                this.interaction.confirm_new = false;
                                cx.notify();
                            }),
                        )),
                )
            })
            .when_some(problem.clone(), |composer, problem| {
                composer.child(
                    div()
                        .debug_selector(|| "ask-problem".into())
                        .text_color(rgb(self.text_ink(self.theme.muted)))
                        .child(problem),
                )
            })
            .when(loading || working, |composer| {
                composer.child(
                    div()
                        .debug_selector(|| "ask-progress".into())
                        .text_color(rgb(self.text_ink(self.theme.muted)))
                        .child(if loading {
                            fl!("ask-loading")
                        } else {
                            fl!("ask-working")
                        }),
                )
            })
            .child(
                div()
                    .p_3()
                    .rounded(px(corners::PANEL))
                    .border_1()
                    .border_color(rgb(self.theme.active))
                    .bg(rgb(self.theme.surface))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(self.input.clone())
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .when(working, |actions| {
                                actions.child(
                                    self.button("ask-stop", fl!("ask-stop"), true).on_click(
                                        cx.listener(|this, _, _, cx| {
                                            this.owner.update(cx, |owner, cx| owner.stop(cx))
                                        }),
                                    ),
                                )
                            })
                            .child(self.button("ask-send", fl!("ask-send"), send).on_click(
                                cx.listener(move |this, _, _, cx| {
                                    if send {
                                        this.submit(cx);
                                    }
                                }),
                            )),
                    ),
            );
        #[cfg(all(feature = "integration-test", target_os = "macos"))]
        let composer = crate::smoke::ask_provider::control(
            "ask-provider-composer".into(),
            composer.id("ask-provider-composer"),
        );
        div()
            .id("ask-spaceport")
            .track_focus(&self.reader_focus)
            .debug_selector(|| "ask-spaceport".into())
            .size_full()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .text_font(&self.config.ui)
            .text_size(px(self.config.ui.size))
            .text_color(rgb(self.theme.foreground))
            .bg(rgb(self.theme.background))
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key != "enter"
                    || !this.input.read(cx).focus.is_focused(window)
                    || this.input.read(cx).is_composing()
                {
                    return;
                }
                let modifiers = event.keystroke.modifiers;
                if modifiers.shift && !modifiers.control && !modifiers.alt && !modifiers.platform {
                    this.input.update(cx, |input, cx| {
                        input.replace_text_in_range(None, "\n", window, cx)
                    });
                } else if !modifiers.modified() {
                    this.submit(cx);
                } else {
                    return;
                }
                cx.stop_propagation();
                window.prevent_default();
            }))
            .capture_action(cx.listener(|this, _: &crate::actions::Copy, window, cx| {
                if this.reader_focus.is_focused(window) {
                    this.reader_key(&crate::actions::edit_key("c"), window, cx);
                    cx.stop_propagation();
                }
            }))
            .capture_action(
                cx.listener(|this, _: &crate::actions::SelectAll, window, cx| {
                    if this.reader_focus.is_focused(window) {
                        this.reader_key(&crate::actions::edit_key("a"), window, cx);
                        cx.stop_propagation();
                    }
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if event.pressed_button != Some(MouseButton::Left) {
                    this.release_selection_drag();
                    return;
                }
                if !this.selection.is_selecting() {
                    return;
                }
                this.drag.move_to(event.position);
                if let Some(caret) = this.geometry.hit(event.position)
                    && this.selection.drag(this.prepared.text(), caret)
                {
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.release_selection_drag()),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.release_selection_drag()),
            )
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if this.reader_focus.is_focused(window) {
                    this.reader_key(event, window, cx);
                    cx.stop_propagation();
                }
            }))
            .child(header)
            .child(self.glide.watch(&self.list, self.selection.is_selecting()))
            .when(!configured, |page| {
                page.when_some(problem, |page, problem| {
                    page.child(
                        div()
                            .px_6()
                            .py_2()
                            .text_color(rgb(self.text_ink(self.theme.muted)))
                            .child(problem),
                    )
                })
            })
            .child(transcript)
            .when(configured, |page| {
                page.child(div().px_6().pb_4().flex().justify_center().child(composer))
            })
    }
}
