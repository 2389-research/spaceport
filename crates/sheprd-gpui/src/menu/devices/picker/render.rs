// ABOUTME: Shares device picker row layout with passive departure content.
// ABOUTME: Keeps live scrolling, input, and selection handlers separate from captured values.
use super::*;

impl SheprdWindow {
    pub(in crate::menu::devices) fn render_devices_picker_paint(
        &self,
        paint: &PickerPaint,
        cx: &mut Context<Self>,
    ) -> Div {
        let list = div()
            .id("devices-list")
            .debug_selector(|| "devices-list".into())
            .w_full()
            .h(paint.height)
            .flex_none()
            .relative()
            .overflow_hidden()
            .children(paint.blocks.iter().map(|block| {
                let content =
                    match &block.content {
                        paint::Content::Header => self.devices_list_header().into_any_element(),
                        paint::Content::Device(device) => {
                            self.device_row_content(device, None, cx).into_any_element()
                        }
                        paint::Content::Sessions(group) => {
                            let rows = div().flex().flex_col().gap(px(4.)).children(
                                group.rows.iter().map(|row| {
                                    let element = match row {
                                        paint::SessionLine::Row(row) => {
                                            self.render_session_row_paint(row, cx)
                                        }
                                        paint::SessionLine::Note(note) => {
                                            self.render_session_note_paint(note)
                                        }
                                    };
                                    session_indent(&self.theme).child(element)
                                }),
                            );
                            crate::motion::disclosure(rows, Some((block.bounds.size.height, 1.)))
                                .opacity(group.opacity)
                                .into_any_element()
                        }
                        paint::Content::Core(catalog) => self
                            .render_core_machine_catalog_paint(catalog, cx)
                            .into_any_element(),
                    };
                let index = block.index;
                div()
                    .id(("devices-block", index))
                    .debug_selector(move || format!("devices-block-{index}"))
                    .absolute()
                    .top(block.bounds.top())
                    .left(block.bounds.left())
                    .w(block.bounds.size.width)
                    .h(block.bounds.size.height)
                    .overflow_hidden()
                    .child(content)
            }));
        div()
            .flex()
            .flex_col()
            .child(self.render_activity_card_paint(&paint.activity, cx))
            .child(list)
    }

    fn devices_list_header(&self) -> Div {
        div()
            .p(px(8.))
            .flex()
            .items_center()
            .justify_between()
            .gap(px(8.))
            .text_color(rgb(self.theme.muted))
            .child(div().flex_none().child(fl!("devices-heading")))
            // Wraps rather than cutting a longer language short.
            .child(
                div()
                    .min_w_0()
                    .text_size(px(self.config.ui.size * 0.85))
                    .child(multi_select_hint()),
            )
    }

    pub(in crate::menu) fn render_devices(&self, cx: &mut Context<Self>) -> Div {
        if let Some(child) = self.menu.devices_reveal.get() {
            if self.menu.devices_scroll.bounds().size.height > px(0.) {
                self.menu.devices_scroll.scroll_to_item(child);
                // The sessions grow open: follow them until they are open.
                let opening = self.menu.unfolded_since.is_some_and(|since| {
                    crate::motion::progress(
                        since,
                        crate::motion::now(),
                        crate::motion::duration(crate::motion::ENTER),
                    )
                    .is_some()
                });
                if !opening {
                    self.menu.devices_reveal.set(None);
                }
            } else {
                // Laid out for the first time this frame: reveal on the next.
                cx.notify();
            }
        }
        let mut view = div()
            .id("devices-list")
            .debug_selector(|| "devices-list".into())
            .max_h(device_list_height(self.menu.anchor.y))
            .overflow_y_scroll()
            .track_scroll(&self.menu.devices_scroll)
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(self.devices_list_header());
        let mut ordinal = 0;
        let mut session_row = 0;
        // The unfolded device's session lines, which open as one block under
        // their device rather than each popping in on its own.
        let mut unfolded: Vec<AnyElement> = Vec::new();
        for line in self.picker_lines() {
            let selected = self.menu.selected == Some(ordinal);
            let element = match &line {
                Line::Session(Entry::Row { row, spec }) => {
                    let element =
                        self.sessions_row(session_row, ordinal, row.clone(), selected, spec, cx);
                    session_row += 1;
                    Some(element)
                }
                Line::Session(entry) => self.session_note(entry),
                _ => self.device_row(&line).map(|index| {
                    self.render_device_row(&line, index, ordinal, selected, cx)
                        .into_any_element()
                }),
            };
            if let Some(element) = element {
                match line {
                    Line::Session(_) => {
                        unfolded.push(
                            session_indent(&self.theme)
                                .child(element)
                                .into_any_element(),
                        );
                    }
                    _ => {
                        view = self.unfolded_sessions(view, &mut unfolded);
                        view = view.child(element);
                        if let Line::Device(index) = line {
                            view = self.folding_sessions(view, index, cx);
                        }
                    }
                }
            }
            if line.selectable() {
                ordinal += 1;
            }
        }
        view = self.unfolded_sessions(view, &mut unfolded);
        view = view.children(self.render_core_machine_catalog(cx));
        // The card stays put above the list, which scrolls on its own.
        div()
            .flex()
            .flex_col()
            .child(self.render_activity_card(cx))
            .child(view)
    }

    /// The unfolded device's session lines, as one block that grows open
    /// under its device; nothing when no device is unfolded. The id follows
    /// the device, so unfolding another plays the opening again.
    fn unfolded_sessions(&self, view: Stateful<Div>, lines: &mut Vec<AnyElement>) -> Stateful<Div> {
        if lines.is_empty() {
            return view;
        }
        let device = self.menu.unfolded_device.clone().unwrap_or_default();
        let rows = self.menu.unfolded_rows.clone();
        let content = div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .on_children_prepainted(move |bounds, _, _| *rows.borrow_mut() = bounds)
            .children(lines.drain(..));
        #[cfg(test)]
        if self.menu.appear.live {
            let measured = self.menu.unfolded_height.clone();
            return view.child(
                crate::motion::disclosure(
                    content,
                    Some((
                        measured.get(),
                        self.picker_unfold_fraction(crate::motion::now()),
                    )),
                )
                .on_children_prepainted(move |bounds, _, _| {
                    if let Some(content) = bounds.first() {
                        measured.set(content.size.height);
                    }
                }),
            );
        }
        view.child(crate::motion::unfold(
            content,
            ElementId::Name(format!("device-sessions-{device}").into()),
            self.menu.unfolded_height.clone(),
            self.menu.unfolded_since.unwrap_or_else(crate::motion::now),
            self.menu.unfold_from,
        ))
    }

    /// The sessions of device `index` while they close after it folded.
    /// They are only a picture of what was there: nothing in them takes the
    /// pointer, and the keys never reach them.
    fn folding_sessions(
        &self,
        view: Stateful<Div>,
        index: usize,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        // Settled unit fixtures omit the fold; explicit fraction fixtures share its frame.
        let Some((folding, _)) = self.picker_folding(index, crate::motion::now()) else {
            return view;
        };
        let mut lines = div().relative().flex().flex_col().gap(px(4.));
        let mut row = 0;
        for entry in self.device_session_entries(index) {
            let element = match &entry {
                Entry::Row { row: choice, spec } => {
                    row += 1;
                    self.sessions_row(row - 1, usize::MAX, choice.clone(), false, spec, cx)
                }
                _ => match self.session_note(&entry) {
                    Some(note) => note,
                    None => continue,
                },
            };
            lines = lines.child(session_indent(&self.theme).child(element));
        }
        let content = lines.child(
            div()
                .id("device-sessions-folding-cover")
                .absolute()
                .inset_0()
                .occlude(),
        );
        #[cfg(test)]
        if self.menu.appear.live {
            let (_, shown) = self
                .picker_folding(index, crate::motion::now())
                .unwrap_or((folding, 0.));
            return view.child(crate::motion::disclosure(
                content,
                Some((folding.height, shown)),
            ));
        }
        view.child(crate::motion::fold(
            content,
            ElementId::Name(format!("device-sessions-folding-{}", folding.device).into()),
            folding.height,
            folding.since,
        ))
    }

    /// All Devices, one device, or Add Device. `index` is the row's place in
    /// [`Self::choose_device`] and names its selectors.
    fn render_device_row(
        &self,
        line: &Line,
        index: usize,
        ordinal: usize,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let paint = self.device_row_paint(line, index, selected);
        let device = match line {
            Line::Device(device) => Some(*device),
            _ => None,
        };
        self.device_row_content(&paint, Some((ordinal, device)), cx)
    }

    fn device_row_content(
        &self,
        paint: &paint::DevicePaint,
        action: Option<(usize, Option<usize>)>,
        cx: &mut Context<Self>,
    ) -> Div {
        let index = paint.index;
        let selected = paint.selected;
        let checked = paint.checked;
        let enabled = paint.enabled;
        let leading = paint.leading;
        let active = action.is_some();
        let row = div()
            .id(("device-row", index))
            .debug_selector(move || format!("device-row-{index}"))
            .p(px(8.))
            .rounded(px(crate::config::corners::CONTROL))
            .flex_none()
            .flex()
            .items_center()
            .gap(px(8.))
            .when(selected, |s| s.bg(rgb(self.theme.active)))
            // The current scope is highlighted as a session row marks its
            // current session, rather than with a trailing check.
            .when(checked, |s| s.bg(rgb(self.theme.primary_wash())))
            .when(active && enabled, |s| s.cursor_pointer())
            .text_color(rgb(if enabled {
                self.theme.foreground
            } else {
                self.theme.muted
            }))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap(px(8.))
                            .when_some(leading, |row, path| {
                                row.child(svg().path(path).size(px(16.)).flex_none().text_color(
                                    rgb(if enabled {
                                        self.theme.foreground
                                    } else {
                                        self.theme.muted
                                    }),
                                ))
                            })
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .when(checked, |label| {
                                        label
                                            .debug_selector(move || {
                                                format!("device-current-{index}")
                                            })
                                            .font_weight(FontWeight::SEMIBOLD)
                                    })
                                    .child(paint.label.clone()),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(self.config.ui.size * 0.85))
                            .text_color(rgb(self.theme.muted))
                            .child(paint.detail.clone()),
                    ),
            )
            .when_some(paint.online, |row, online| {
                row.child(
                    div()
                        .debug_selector(move || format!("device-dot-{index}"))
                        .size(px(7.))
                        .flex_none()
                        .rounded_full()
                        .bg(rgb(if online {
                            colors::online(&self.theme)
                        } else {
                            self.theme.muted
                        })),
                )
            })
            .when_some(paint.chip.as_ref(), |row, chip| {
                row.child(self.render_session_chip(
                    chip,
                    index,
                    action.and_then(|(_, device)| device),
                    cx,
                ))
            })
            .when_some(action, |row, (ordinal, _)| {
                row.on_hover(cx.listener(move |this, hovered, _, cx| {
                    if *hovered {
                        this.menu.selected = Some(ordinal);
                        cx.notify();
                    }
                }))
                .on_click(cx.listener(
                    move |this, event: &ClickEvent, window, cx| {
                        if crate::keymap::secondary(&event.modifiers()) {
                            this.toggle_listed_device(index, window, cx);
                        } else {
                            this.choose_device(index, window, cx);
                        }
                    },
                ))
            });
        div().flex_none().child(row)
    }

    /// The session a device is on, which unfolds the device's other sessions
    /// beneath it and folds them away again.
    fn render_session_chip(
        &self,
        paint: &paint::ChipPaint,
        index: usize,
        device: Option<usize>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let theme = &self.theme;
        div()
            .id(("device-sessions", index))
            .debug_selector(move || format!("device-sessions-{index}"))
            .when(device.is_some(), |chip| {
                chip.role(Role::Button)
                    .aria_label(paint.spoken.clone())
                    .cursor_pointer()
                    .hover(|style| style.bg(colors::action_hover(theme)))
            })
            .flex_none()
            .max_w(px(96.))
            .flex()
            .items_center()
            .gap(px(4.))
            .pl(px(6.))
            .pr(px(4.))
            .py(px(2.))
            .rounded(px(crate::config::corners::SMALL))
            .text_size(px(self.config.ui.size * 0.85))
            .text_color(rgb(theme.subtext()))
            .child(match &paint.label {
                None => svg()
                    .path("icons/sessions.svg")
                    .size(px(12.))
                    .flex_none()
                    .text_color(rgb(theme.subtext()))
                    .into_any_element(),
                Some(label) => div()
                    .min_w_0()
                    .truncate()
                    .child(label.clone())
                    .into_any_element(),
            })
            .child(
                svg()
                    .path(if paint.unfolded {
                        "icons/chevron-up.svg"
                    } else {
                        "icons/chevron-down.svg"
                    })
                    .size(px(10.))
                    .flex_none()
                    .text_color(rgb(theme.muted)),
            )
            .when_some(device, |chip, device| {
                chip.on_click(cx.listener(move |this, _, _, cx| {
                    cx.stop_propagation();
                    this.toggle_unfolded(device, cx);
                }))
            })
    }
}
