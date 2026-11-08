//! The Add Device dialog's frame: one title, a rail of what can be added (this
//! platform's own hosts), and the chosen source's form beside it. Each source keeps its own form state,
//! so switching back and forth does not lose what was typed.

use super::super::Page;
use crate::SheprdWindow;
use crate::i18n::fl;
use gpui::{prelude::*, *};

/// Below this panel width the rail shows only icons.
pub(in crate::menu) const COMPACT_BELOW: f32 = 560.;
/// The dialog's width when there is room: the rail and a form beside it.
pub(in crate::menu) const WIDTH: f32 = 640.;
/// The dialog's height, the same for every source so switching never
/// resizes it: room for the SSH form and setup prompts. A form taller
/// than that scrolls inside.
pub(in crate::menu) const HEIGHT: f32 = 620.;

/// One entry of the rail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::menu) enum Source {
    /// SSH hosts.
    Host,
    /// WSL distributions on this machine, listed only on Windows.
    Wsl,
}

impl Source {
    /// The rail entry a page belongs to, if it is part of Add Device.
    pub(in crate::menu) fn of(page: Page) -> Option<Self> {
        match page {
            Page::AddDevice => Some(Self::Host),
            Page::AddWsl => Some(Self::Wsl),
            _ => None,
        }
    }

    fn name(self) -> String {
        match self {
            Self::Host => fl!("add-device-source-host"),
            // A product's name.
            Self::Wsl => "WSL".to_owned(), // i18n: data
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Host => "icons/terminal.svg",
            Self::Wsl => "icons/computer.svg",
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::Host => "add-device-source-host",
            Self::Wsl => "add-device-source-wsl",
        }
    }
}

/// The Devices picker's line under "Add Device…".
pub(super) fn add_device_offer() -> String {
    if cfg!(windows) {
        fl!("add-device-offer-ssh-wsl")
    } else {
        fl!("add-device-offer-ssh")
    }
}

impl SheprdWindow {
    /// This platform's own hosts.
    pub(in crate::menu) fn rail_sources(&self) -> Vec<Source> {
        let mut sources = vec![Source::Host];
        if cfg!(windows) {
            sources.push(Source::Wsl);
        }
        sources
    }

    /// Show `source`'s form, reopening the one it had if it is still there.
    pub(in crate::menu) fn open_source(
        &mut self,
        source: Source,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.menu.page.and_then(Source::of) == Some(source) {
            return;
        }
        self.menu.error = None;
        match source {
            Source::Wsl => {
                if self.menu.wsl_setup.is_some() {
                    self.menu.page = Some(Page::AddWsl);
                    window.focus(&self.menu.focus.clone(), cx);
                } else {
                    self.open_add_wsl(window, cx);
                }
            }
            Source::Host => match &self.menu.device_setup {
                Some(setup) => {
                    let focus = setup.fields[0].read(cx).focus.clone();
                    self.menu.page = Some(Page::AddDevice);
                    window.focus(&focus, cx);
                }
                None => self.open_add_device(window, cx),
            },
        }
        cx.notify();
    }

    /// Keyboard access to the rail: Ctrl-Tab and Ctrl-Shift-Tab step through
    /// it from any field, wrapping at the ends. Returns whether the key was
    /// used. Cmd or Ctrl with a digit stay with the app-wide shortcuts.
    pub(in crate::menu) fn add_device_rail_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let sources = self.rail_sources();
        let count = sources.len();
        if count < 2 {
            return false;
        }
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;
        let current = self
            .menu
            .page
            .and_then(Source::of)
            .and_then(|shown| sources.iter().position(|source| *source == shown))
            .unwrap_or(0);
        if key != "tab" || !modifiers.control || modifiers.alt || modifiers.platform {
            return false;
        }
        let index = if modifiers.shift {
            (current + count - 1) % count
        } else {
            (current + 1) % count
        };
        let Some(&source) = sources.get(index) else {
            return false;
        };
        self.open_source(source, window, cx);
        true
    }

    /// The whole dialog: the shared header, the rail, and `pane`, which draws
    /// its own body and footer. `selector` and `close` keep each source's
    /// own debug names for its frame and Close button.
    pub(in crate::menu) fn add_device_frame(
        &self,
        selector: &'static str,
        close: &'static str,
        compact: bool,
        pane: impl IntoElement,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let header = div()
            .debug_selector(|| "device-setup-header".into())
            .flex_none()
            .p(px(16.))
            .border_b_1()
            .border_color(rgb(theme.active))
            .flex()
            .items_center()
            .gap(px(12.))
            .child(
                div()
                    .flex_1()
                    .text_size(px(self.config.ui.size * 1.35))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(fl!("add-device-title")),
            )
            .child(
                div()
                    .id(close)
                    .debug_selector(move || close.into())
                    .px_2()
                    .py_1()
                    .cursor_pointer()
                    .rounded(px(crate::config::corners::CONTROL))
                    .hover(|s| s.bg(rgb(theme.active)))
                    .child(fl!("add-device-close"))
                    .on_click(cx.listener(|this, _, window, cx| this.dismiss_menu(window, cx))),
            );
        let sources = self.rail_sources();
        let body = div().flex_1().min_h_0().flex();
        // A lone source needs no rail.
        let body = if sources.len() > 1 {
            body.child(self.render_add_device_rail(&sources, compact, cx))
        } else {
            body
        };
        div()
            .debug_selector(move || selector.into())
            .flex_1()
            .flex()
            .flex_col()
            .min_h_0()
            .child(header)
            .child(
                body.child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .child(pane),
                ),
            )
    }

    fn render_add_device_rail(
        &self,
        sources: &[Source],
        compact: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = &self.theme;
        let current = self.menu.page.and_then(Source::of);
        // A provider's pane opened from elsewhere belongs under Add provider
        // while the rail omits it.
        let mut rail = div()
            .debug_selector(|| "add-device-rail".into())
            .flex_none()
            // Wide enough for its names in any language, up to a limit.
            .when(compact, |rail| rail.w(px(52.)))
            .when(!compact, |rail| rail.min_w(px(172.)).max_w(px(240.)))
            .p(px(8.))
            .flex()
            .flex_col()
            .gap(px(2.))
            .border_r_1()
            .border_color(rgb(theme.active));
        for &source in sources {
            let on = current == Some(source);
            let id = source.id();
            let icon = div()
                .flex_none()
                .size(px(24.))
                .rounded(px(crate::config::corners::SMALL))
                .bg(rgb(theme.active))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    svg()
                        .path(source.icon())
                        .size(px(14.))
                        .text_color(rgb(theme.foreground)),
                );
            rail = rail.child(
                div()
                    .id(id)
                    .debug_selector(move || id.into())
                    .p(px(6.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .rounded(px(crate::config::corners::CONTROL))
                    .cursor_pointer()
                    .when(on, |row| row.bg(rgb(theme.primary_wash())))
                    .when(!on, |row| row.hover(|s| s.bg(rgb(theme.active))))
                    .child(icon)
                    .when(!compact, |row| {
                        // Its own width counts, so the rail widens to fit it.
                        row.child(
                            div()
                                .flex_grow(1.)
                                .min_w_0()
                                .truncate()
                                .child(source.name()),
                        )
                    })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_source(source, window, cx);
                    })),
            );
        }
        rail
    }
}
