//! The Add Device dialog's sources: which of them this platform offers, the
//! words naming each, and the page each form belongs to. The frame around the
//! chosen one is `spaceport_menu::devices::dialog`'s; opening a source and
//! stepping through the rail are the window's.

use super::super::Page;
use crate::SpaceportWindow;
use crate::i18n::fl;
use gpui::{prelude::*, *};
use spaceport_menu::devices::dialog;

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

impl SpaceportWindow {
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

    /// The whole dialog, as `spaceport_menu` paints it: the shared header, the
    /// rail of this platform's sources, and `pane`, which draws its own body
    /// and footer. `selector` and `close` keep each source's own debug names
    /// for its frame and Close button.
    pub(in crate::menu) fn add_device_frame(
        &self,
        selector: &'static str,
        close: &'static str,
        compact: bool,
        pane: impl IntoElement,
        cx: &mut Context<Self>,
    ) -> Div {
        let sources = self.rail_sources();
        let current = self.menu.page.and_then(Source::of);
        let frame = dialog::Frame {
            dialog: selector.to_owned(),
            close: close.to_owned(),
            title: fl!("add-device-title"),
            close_label: fl!("add-device-close"),
            sources: sources
                .iter()
                .map(|&source| dialog::Source {
                    selector: source.id().to_owned(),
                    icon: source.icon().to_owned(),
                    label: source.name(),
                    current: current == Some(source),
                })
                .collect(),
            compact,
        };
        let text = crate::menu::chrome::text(&self.config.ui);
        dialog::frame(
            &text,
            &self.theme,
            frame,
            pane,
            move |action, control| match action {
                dialog::Action::Close => control
                    .on_click(cx.listener(|this, _, window, cx| this.dismiss_menu(window, cx))),
                dialog::Action::Source(index) => match sources.get(index) {
                    Some(&source) => {
                        control.on_click(cx.listener(move |this, _, window, cx| {
                            this.open_source(source, window, cx)
                        }))
                    }
                    None => control,
                },
            },
        )
    }
}
