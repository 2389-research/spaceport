// ABOUTME: Previews and commits theme choices without replacing newer UI intents.
// ABOUTME: Refreshes preview palettes when OS contrast preferences change.
// i18n: migrated
use crate::{
    SheprdWindow,
    config::{Theme, ThemeName},
    i18n::fl,
    menu::Page,
    search_input::SearchInput,
};
use gpui::{prelude::*, *};

mod input;
mod paint;
mod render;
use input::Input;
use paint::Content;
pub(crate) use paint::Paint;

pub(super) struct ThemePicker {
    input: Option<Input>,
    names: Vec<String>,
    pub(super) filtered: Vec<String>,
    /// How many rows of `filtered` are starred themes, which lead the list.
    favorites: usize,
    // One favorites write at a time, so a quick run of stars cannot land
    // out of order; stars clicked meanwhile wait in `favorite_edits`.
    favorites_saving: bool,
    favorite_edits: Vec<crate::config::favorite_themes::FavoriteEdit>,
    /// The last favorites write's failure, until a later one succeeds.
    favorites_error: Option<String>,
    selected: usize,
    scroll: UniformListScrollHandle,
    error: Option<String>,
    query: String,
    discovering: bool,
    baseline: Option<Theme>,
    session: u64,
    request: u64,
    desired: Option<String>,
    loaded: Option<String>,
    accepting: bool,
    saving: bool,
    // Keep the slot across dismiss/reopen: blocking I/O cannot be cancelled by
    // dropping a GPUI task. Only its completion may release the slot.
    in_flight: Option<(u64, u64)>,
    window: AnyWindowHandle,
    status_scroll: ScrollHandle,
}

impl ThemePicker {
    fn filter(&mut self, query: &str, favorites: &[String]) {
        self.query = query.to_owned();
        self.group(favorites);
        self.selected = 0;
        self.scroll.scroll_to_item(0, ScrollStrategy::Top);
    }

    /// Lists the themes matching the query, starred ones first, each group
    /// in catalog order.
    fn group(&mut self, favorites: &[String]) {
        let query = self.query.trim().to_lowercase();
        let (mut filtered, rest): (Vec<_>, Vec<_>) = self
            .names
            .iter()
            .filter(|name| name.to_lowercase().contains(&query))
            .cloned()
            .partition(|name| favorites.contains(name));
        self.favorites = filtered.len();
        filtered.extend(rest);
        self.filtered = filtered;
    }
}

impl SheprdWindow {
    pub(super) fn open_theme_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if crate::settings_window::theme_pending(cx) {
            return;
        }
        if !self.open_menu(window, cx) {
            return;
        }
        // A pending reload must not replace this newer interactive appearance.
        self.config_load = None;
        self.menu.page = Some(Page::Themes);
        let mut picker = if let Some(picker) = self.menu.themes.take() {
            picker
        } else {
            ThemePicker {
                input: None,
                names: Vec::new(),
                filtered: Vec::new(),
                favorites: 0,
                favorites_saving: false,
                favorite_edits: Vec::new(),
                favorites_error: None,
                selected: 0,
                scroll: UniformListScrollHandle::new(),
                error: None,
                query: String::new(),
                discovering: false,
                baseline: None,
                session: 0,
                request: 0,
                desired: None,
                loaded: None,
                accepting: false,
                saving: false,
                in_flight: None,
                window: window.window_handle(),
                status_scroll: ScrollHandle::new(),
            }
        };
        picker.session += 1;
        picker.baseline = Some(self.theme.clone());
        picker.desired = None;
        picker.loaded = None;
        picker.accepting = false;
        picker.error = None;
        picker.names = Theme::BUILTIN_NAMES
            .iter()
            .map(|name| (*name).into())
            .collect();
        // A theme that follows the system is picked one side at a time: the
        // side the current appearance shows.
        let current = ThemeName::side(&self.config.theme, crate::app::light_appearance(cx));
        if !picker.names.iter().any(|name| name == current) {
            picker.names.push(current.to_owned());
        }
        picker.names.sort();
        picker.filter("", &self.config.favorite_themes);
        picker.selected = picker
            .filtered
            .iter()
            .position(|name| name == current)
            .unwrap_or(0);
        let input = Input::new(cx);
        input.search.update(cx, |input, cx| {
            input.set_appearance(self.config.ui.clone(), self.theme.clone(), cx);
            window.focus(&input.focus, cx);
        });
        picker.input = Some(input);
        picker.status_scroll.set_offset(Point::default());
        self.menu.themes = Some(picker);
        self.discover_picker_themes(cx);
        cx.notify();
    }

    fn discover_picker_themes(&mut self, cx: &mut Context<Self>) {
        let Some(picker) = &mut self.menu.themes else {
            return;
        };
        if picker.discovering {
            return;
        }
        picker.discovering = true;
        let session = picker.session;
        let config = self.config.clone();
        let discovery = cx
            .background_executor()
            .spawn(async move { config.available_themes().map_err(crate::Error::from) });
        cx.spawn(async move |this, cx| {
            let result = discovery.await;
            let _ = this.update(cx, |this, cx| {
                this.finish_picker_discovery(session, result, cx);
            });
        })
        .detach();
    }

    fn finish_picker_discovery(
        &mut self,
        session: u64,
        result: crate::Result<Vec<String>>,
        cx: &mut Context<Self>,
    ) {
        let Some(picker) = &mut self.menu.themes else {
            return;
        };
        picker.discovering = false;
        if picker.baseline.is_none() {
            return;
        }
        match result {
            Ok(mut names) => {
                // Theme directories are process-wide, not picker-session state.
                // Reuse a running scan on reopen, adding the current explicit selection.
                let current = ThemeName::side(&self.config.theme, crate::app::light_appearance(cx));
                if !names.iter().any(|name| name == current) {
                    names.push(current.to_owned());
                    names.sort_by_cached_key(|name| (name.to_lowercase(), name.clone()));
                }
                let selected = picker.filtered.get(picker.selected).cloned();
                picker.names = names;
                let Some(search) = picker.search() else {
                    return;
                };
                let query = search.read(cx).text().to_owned();
                picker.filter(&query, &self.config.favorite_themes);
                if let Some(index) = picker
                    .filtered
                    .iter()
                    .position(|name| Some(name) == selected.as_ref())
                {
                    picker.selected = index;
                }
            }
            Err(error) if picker.session == session => picker.error = Some(error.to_string()),
            Err(_) => {}
        }
        if !picker.query.is_empty() && picker.desired.is_none() {
            self.preview_picker_selection(cx);
        }
        cx.notify();
    }

    pub(super) fn theme_save_in_flight(&self) -> bool {
        self.menu
            .themes
            .as_ref()
            .is_some_and(|picker| picker.saving)
    }

    /// Refreshes the active preview and its cancel target from raw palettes.
    pub(super) fn refresh_picker_contrast(&mut self, cx: &mut Context<Self>) -> bool {
        if self.menu.page != Some(Page::Themes) {
            return false;
        }
        let Some(picker) = &mut self.menu.themes else {
            return false;
        };
        if picker.baseline.is_none() {
            return false;
        }
        let session = picker.session;
        // A committed disk write keeps its token until its result is reconciled.
        if !picker.saving {
            picker.request += 1;
            picker.loaded = None;
        }
        let config = self.config.clone();
        let light = crate::app::light_appearance(cx);
        let task = cx
            .background_executor()
            .spawn(async move { config.raw_theme(light).map(|theme| (config.theme, theme)) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                let (name, raw) = match result {
                    Ok(loaded) => loaded,
                    Err(error) => {
                        tracing::warn!(%error, "Could not refresh preview contrast");
                        return;
                    }
                };
                let Some(picker) = &mut this.menu.themes else {
                    return;
                };
                if picker.session != session
                    || picker.baseline.is_none()
                    || this.config.theme != name
                {
                    return;
                }
                let theme = raw.styled_for(&this.config);
                picker.baseline = Some(theme.clone());
                if picker.desired.is_none() {
                    this.theme = theme;
                    picker.set_search_appearance(&this.config.ui, &this.theme, cx);
                    crate::log_window::set_appearance(&this.config, &this.theme, cx);
                }
                cx.notify();
            });
        })
        .detach();
        self.drive_picker_load(cx);
        true
    }

    pub(super) fn cancel_theme_preview(&mut self, cx: &mut Context<Self>) -> bool {
        // Starting the disk write is the commit boundary. Its result must be
        // reconciled before another modal, cancellation, or reload can proceed.
        if self.theme_save_in_flight() {
            return false;
        }
        let mut refresh = false;
        if let Some(picker) = &mut self.menu.themes {
            if let Some(theme) = picker.baseline.take() {
                refresh = theme.contrast != self.config.contrast.or_system();
                self.theme = theme;
                crate::log_window::set_appearance(&self.config, &self.theme, cx);
            }
            picker.session += 1;
            picker.desired = None;
            picker.accepting = false;
        }
        if refresh {
            Self::defer_theme_reload(cx);
        }
        true
    }

    fn preview_picker_selection(&mut self, cx: &mut Context<Self>) {
        if self.menu.page != Some(Page::Themes) {
            return;
        }
        let Some(picker) = &mut self.menu.themes else {
            return;
        };
        if picker.accepting {
            return;
        }
        let desired = picker.filtered.get(picker.selected).cloned();
        if picker.desired != desired {
            picker.request += 1;
            picker.desired = desired;
            picker.loaded = None;
            picker.error = None;
            if let Some(theme) = picker
                .desired
                .as_deref()
                .and_then(|name| Theme::builtin(name.trim()))
                .map(|theme| theme.styled_for(&self.config))
            {
                self.theme = theme;
                picker.loaded = picker.desired.clone();
            } else if picker.desired.is_none()
                && let Some(theme) = &picker.baseline
            {
                self.theme = theme.clone();
            }
        }
        picker.set_search_appearance(&self.config.ui, &self.theme, cx);
        crate::log_window::set_appearance(&self.config, &self.theme, cx);
        self.drive_picker_load(cx);
        cx.notify();
    }

    fn apply_picker_theme(&mut self, name: &str, cx: &mut Context<Self>) {
        let Some(picker) = &mut self.menu.themes else {
            return;
        };
        if picker.accepting {
            return;
        }
        if let Some(index) = picker
            .filtered
            .iter()
            .position(|candidate| candidate == name)
        {
            picker.selected = index;
        } else {
            return;
        }
        self.preview_picker_selection(cx);
        if let Some(picker) = &mut self.menu.themes {
            picker.accepting = true;
        }
        self.drive_picker_load(cx);
    }

    fn drive_picker_load(&mut self, cx: &mut Context<Self>) {
        let Some(picker) = &mut self.menu.themes else {
            return;
        };
        if picker.baseline.is_none() || picker.in_flight.is_some() {
            return;
        }
        let Some(name) = picker.desired.clone() else {
            return;
        };
        if picker.loaded.as_ref() == Some(&name) && !picker.accepting {
            return;
        }
        let token = (picker.session, picker.request);
        let saving = picker.accepting && picker.loaded.as_ref() == Some(&name);
        picker.saving = saving;
        picker.in_flight = Some(token);
        let window = picker.window;
        let light = crate::app::light_appearance(cx);
        let mut config = self.config.clone();
        config.theme = ThemeName::with_side(&self.config.theme, light, &name);
        let theme = self.theme.clone();
        let task = cx.background_executor().spawn(async move {
            if saving {
                config.save_theme(&config.theme).map(|()| theme)
            } else {
                config.raw_theme(light)
            }
            .map_err(crate::Error::from)
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = window.update(cx, |_, window, cx| {
                let _ = this.update(cx, |this, cx| {
                    this.finish_picker_load(token, saving, result, window, cx)
                });
            });
        })
        .detach();
    }

    fn finish_picker_load(
        &mut self,
        token: (u64, u64),
        saving: bool,
        result: crate::Result<Theme>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(picker) = &mut self.menu.themes else {
            return;
        };
        if picker.in_flight != Some(token) {
            return;
        }
        picker.in_flight = None;
        picker.saving = false;
        if picker.baseline.is_none() || token != (picker.session, picker.request) {
            self.drive_picker_load(cx);
            return;
        }
        match result {
            Ok(theme) => {
                let refresh = saving && theme.contrast != self.config.contrast.or_system();
                self.theme = if saving {
                    theme
                } else {
                    theme.styled_for(&self.config)
                };
                picker.loaded = picker.desired.clone();
                picker.set_search_appearance(&self.config.ui, &self.theme, cx);
                if saving {
                    if let Some(name) = &picker.desired {
                        self.config.theme = ThemeName::with_side(
                            &self.config.theme,
                            crate::app::light_appearance(cx),
                            name,
                        );
                    }
                    crate::log_window::set_appearance(&self.config, &self.theme, cx);
                    picker.baseline = None;
                    self.dismiss_menu(window, cx);
                    if refresh {
                        Self::defer_theme_reload(cx);
                    }
                } else {
                    crate::log_window::set_appearance(&self.config, &self.theme, cx);
                    self.drive_picker_load(cx);
                }
            }
            Err(error) => {
                picker.error = Some(error.to_string());
                picker.accepting = false;
            }
        }
        cx.notify();
    }

    pub(super) fn theme_picker_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(picker) = &mut self.menu.themes else {
            return;
        };
        // Unhandled text must reach the native input system, including IME commands.
        if picker
            .search()
            .is_none_or(|search| search.read(cx).is_composing())
        {
            return;
        }
        match event.keystroke.key.as_str() {
            "escape" => {
                cx.stop_propagation();
                window.prevent_default();
                self.dismiss_menu(window, cx);
            }
            "up" | "down" if !picker.filtered.is_empty() => {
                cx.stop_propagation();
                window.prevent_default();
                if picker.accepting {
                    return;
                }
                let count = picker.filtered.len();
                picker.selected = (picker.selected
                    + if event.keystroke.key == "up" {
                        count - 1
                    } else {
                        1
                    })
                    % count;
                picker
                    .scroll
                    .scroll_to_item(picker.selected, ScrollStrategy::Center);
                self.preview_picker_selection(cx);
            }
            "enter" => {
                cx.stop_propagation();
                window.prevent_default();
                if let Some(name) = picker.filtered.get(picker.selected).cloned() {
                    self.apply_picker_theme(&name, cx);
                }
            }
            _ => {}
        }
    }

    pub(super) fn render_theme_picker(&self, cx: &mut Context<Self>) -> Div {
        let Some(picker) = &self.menu.themes else {
            return div();
        };
        if picker.search().is_none() {
            return div();
        }
        self.render_theme_picker_content(Content::Live(picker), cx)
    }

    pub(crate) fn render_theme_picker_paint(&self, paint: &Paint, cx: &mut Context<Self>) -> Div {
        self.render_theme_picker_content(Content::Frozen(paint), cx)
    }

    fn render_theme_picker_content(&self, picker: Content<'_>, cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        let font = &self.config.ui;
        div()
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .child(
                div()
                    .flex_none()
                    .p(px(16.))
                    .border_b_1()
                    .border_color(rgb(theme.active))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(12.))
                            .child(
                                div()
                                    .flex_1()
                                    .text_size(px(font.size * 1.35))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(fl!("theme-picker-title")),
                            )
                            .child(
                                div()
                                    .id("theme-close")
                                    .debug_selector(|| "theme-close".into())
                                    .px_2()
                                    .py_1()
                                    .rounded(px(crate::config::corners::CONTROL))
                                    .child(fl!("theme-picker-close"))
                                    .when(picker.active(), |button| {
                                        button
                                            .cursor_pointer()
                                            .hover(|s| s.bg(rgb(theme.active)))
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.dismiss_menu(window, cx)
                                            }))
                                    }),
                            ),
                    )
                    .child(div().pt(px(12.)).children(picker.input()))
                    .child(div().pt(px(8.)).text_color(rgb(theme.muted)).child(fl!(
                        "theme-picker-count",
                        shown = picker.count(),
                        total = picker.total()
                    ))),
            )
            .when(picker.count() == 0, |panel| {
                panel.child(
                    div()
                        .debug_selector(|| "theme-empty".into())
                        .flex_1()
                        .p(px(16.))
                        .text_color(rgb(theme.muted))
                        .child(fl!("theme-picker-empty")),
                )
            })
            .when(picker.count() != 0, |panel| {
                let rows = match picker {
                    Content::Live(live) => uniform_list(
                        "theme-results",
                        live.filtered.len(),
                        cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                            let Some(picker) = &this.menu.themes else {
                                return Vec::new();
                            };
                            range
                                .filter_map(|index| {
                                    let row = this.theme_picker_row_data(picker, index, cx)?;
                                    Some(this.theme_picker_row(
                                        &row,
                                        px(this.config.ui.line_height() + 20.),
                                        true,
                                        cx,
                                    ))
                                })
                                .collect()
                        }),
                    )
                    .track_scroll(&live.scroll)
                    .flex_1()
                    .min_h_0()
                    .into_any_element(),
                    Content::Frozen(paint) => div()
                        .id("theme-results")
                        .flex_1()
                        .min_h_0()
                        .relative()
                        .overflow_hidden()
                        .children(paint.rows.iter().enumerate().map(|(slot, row)| {
                            div()
                                .absolute()
                                .top(paint.offset + paint.row_height * slot as f32)
                                .left_0()
                                .right_0()
                                .child(self.theme_picker_row(row, paint.row_height, false, cx))
                        }))
                        .into_any_element(),
                };
                panel.child(rows)
            })
            .child(
                div()
                    .id("theme-status")
                    .debug_selector(|| "theme-status".into())
                    .flex_none()
                    .h(px(font.line_height() * 3. + 20.))
                    .when(picker.active(), |status| {
                        status.overflow_y_scroll().map(|status| match picker {
                            Content::Live(live) => status.track_scroll(&live.status_scroll),
                            Content::Frozen(_) => status,
                        })
                    })
                    .when(!picker.active(), |status| status.overflow_hidden())
                    .px(px(16.))
                    .py(px(10.))
                    .border_t_1()
                    .border_color(rgb(theme.active))
                    .text_color(rgb(theme.muted))
                    .child(
                        div()
                            .mt(match picker {
                                Content::Live(_) => px(0.),
                                Content::Frozen(paint) => paint.status_offset,
                            })
                            .child(picker.status()),
                    ),
            )
    }
}

mod favorites;

#[cfg(test)]
pub(crate) mod tests;
