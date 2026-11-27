// ABOUTME: Opens and paints popup pages with shared window geometry.
// ABOUTME: Keeps popup input and focus isolated from terminal panes.
//! The popup shell's window half: opening and dismissing it, keeping its input
//! isolated from the panes beneath, and what the page bodies and handlers it
//! draws are. The geometry and the paint are `spaceport_menu::chrome`'s; this
//! is what only the window knows.

// i18n: migrated
mod keys;
mod render;

use super::Page;
use crate::SpaceportWindow;
use crate::i18n::fl;
use crate::{config::FontConfig, fonts::StyledFont};
use gpui::{prelude::*, *};
use spaceport_menu::chrome::{Metrics, Text};

impl SpaceportWindow {
    pub(crate) fn open_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.shift_taps.cancel();
        if !self.cancel_theme_preview(cx) {
            return false;
        }
        self.menu.reset();
        self.apply_shared_theme(cx);
        self.menu.endpoint_target = (
            self.selection_epoch,
            self.endpoints[self.selected_endpoint].generation,
        );
        self.menu.page = Some(Page::Menu);
        self.discard_composition(cx);
        window.focus(&self.menu.focus, cx);
        cx.notify();
        true
    }

    pub(crate) fn dismiss_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.cancel_theme_preview(cx) {
            return;
        }
        // Whatever the pointer was resting on, this dismissal ends that intent.
        self.hover = None;
        self.hover_menu = None;
        self.dismiss_release_notes();
        // Closing the offer without an answer is a "not now": it asks once.
        if self.menu.page == Some(Page::AgentSkill)
            && crate::agent_skill::AgentSkill::choice(cx).is_none()
        {
            crate::agent_skill::AgentSkill::choose(crate::agent_skill::Choice::Declined, cx);
        }
        let return_focus = self
            .menu
            .application
            .as_ref()
            .and_then(|menu| menu.return_focus.clone());
        // A page that animates out keeps what it reads from the reset.
        if let Some(page) = self.menu.page.filter(|page| super::appear::leaves(*page)) {
            let kept = super::appear::Kept::take(&mut self.menu);
            self.menu.appear.state.keep(page, kept);
        }
        self.menu.reset();
        self.apply_shared_theme(cx);
        window.focus(return_focus.as_ref().unwrap_or(&self.focus), cx);
        cx.notify();
    }

    pub(crate) fn restore_menu_focus(&self, window: &mut Window, cx: &mut App) {
        if self.menu.page.is_none() && self.menu.focus.is_focused(window) {
            window.focus(&self.focus, cx);
        }
    }

    pub(crate) fn menu_target_current(&self) -> bool {
        self.menu.endpoint_target
            == (
                self.selection_epoch,
                self.endpoints[self.selected_endpoint].generation,
            )
    }

    /// Asks the selected daemon to reread its config file.
    pub(crate) fn reload_daemon_config(&mut self) {
        let error = self.endpoints[self.selected_endpoint]
            .reload_config()
            .err()
            .map(|error| {
                fl!(
                    "menu-chrome-reload-config-failed",
                    error = error.to_string()
                )
            });
        match error {
            Some(error) => self
                .notices
                .show(crate::notice::Cause::Configuration, error),
            None => self.notices.clear(),
        }
    }

    pub(super) fn menu_items(&self) -> Vec<Item> {
        let mut items = vec![
            Item::Settings,
            Item::Shortcuts,
            Item::Themes,
            Item::IncreaseFontSize,
            Item::DecreaseFontSize,
            Item::ResetFontSize,
            Item::CommandPalette,
            Item::Workspaces,
            Item::ReloadGuiConfig,
            Item::About,
        ];
        if self.live.status.is_connected() {
            items.push(Item::ReloadDaemonConfig);
        }
        items.extend(self.release_notes_item());
        items.push(
            if self.endpoints[self.selected_endpoint]
                .connection
                .handle
                .is_some()
            {
                Item::Detach
            } else {
                Item::Reconnect
            },
        );
        items
    }

    pub(super) fn activate_menu(
        &mut self,
        item: Item,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use crate::config::FONT_SIZE_STEP;
        match item {
            Item::About => self.open_about(window, cx),
            Item::Settings => self.open_preferences(window, cx),
            Item::Shortcuts => self.open_keybinds(window, cx),
            Item::Themes => self.open_theme_picker(window, cx),
            // `command` refuses to act while a page is open, so the font
            // size rows apply here.
            Item::IncreaseFontSize => {
                self.set_terminal_font_size(self.config.terminal.size + FONT_SIZE_STEP, cx);
                self.dismiss_menu(window, cx);
            }
            Item::DecreaseFontSize => {
                self.set_terminal_font_size(self.config.terminal.size - FONT_SIZE_STEP, cx);
                self.dismiss_menu(window, cx);
            }
            Item::ResetFontSize => {
                self.set_terminal_font_size(self.configured_terminal_size, cx);
                self.dismiss_menu(window, cx);
            }
            Item::CommandPalette => self.open_palette(crate::palette::Filter::All, window, cx),
            Item::Workspaces => self.open_palette(crate::palette::Filter::Navigation, window, cx),
            Item::UpdateReady | Item::WhatsNew => self.menu.page = Some(Page::Update),
            Item::ReloadGuiConfig => self.reload_gui_config(window, cx),
            Item::ReloadDaemonConfig => {
                self.reload_daemon_config();
                self.dismiss_menu(window, cx);
            }
            Item::Detach => {
                self.detach_endpoint(cx);
                self.dismiss_menu(window, cx);
            }
            Item::Reconnect => {
                self.reconnect(cx);
                self.dismiss_menu(window, cx);
            }
        }
        cx.notify();
    }

    pub(crate) fn render_menu(&self, window: &Window, cx: &mut Context<Self>) -> Stateful<Div> {
        if self.menu.page == Some(Page::Application) {
            return self.render_application_menu(window, cx);
        }
        if self.menu.page == Some(Page::Devices) && self.menu.session_edit.is_some() {
            let theme = &self.theme;
            let font = &self.config.ui;
            let picker = self
                .anchor_footer_panel(
                    div()
                        .id("session-picker-underlay")
                        .debug_selector(|| "session-picker-underlay".into()),
                    window.viewport_size(),
                    Page::Devices,
                    px(0.),
                )
                .overflow_y_scroll()
                .p(px(6.))
                .rounded(px(crate::config::corners::PANEL))
                .border_1()
                .border_color(rgb(theme.active))
                .bg(rgb(theme.surface))
                .text_color(rgb(theme.foreground))
                .text_font(font)
                .text_size(px(font.size))
                .line_height(px(font.line_height()))
                .child(self.render_devices(cx));
            // The picker remains visible in its original position beneath the
            // modal's dimmed, input-occluding layer. Only that top layer owns focus.
            return div()
                .id("session-menu-stack")
                .absolute()
                .inset_0()
                .child(picker)
                .child(self.render_menu_layer(
                    self.menu.page.unwrap_or(Page::Menu),
                    None,
                    window,
                    cx,
                ));
        }
        self.render_menu_layer(self.menu.page.unwrap_or(Page::Menu), None, window, cx)
    }

    /// Places a footer menu beside its anchor, `lift` short of it while it
    /// slides into place.
    fn anchor_footer_panel(
        &self,
        panel: Stateful<Div>,
        viewport: Size<Pixels>,
        page: Page,
        lift: Pixels,
    ) -> Stateful<Div> {
        spaceport_menu::chrome::anchor_footer_panel(
            panel,
            page,
            self.menu.anchor,
            viewport,
            crate::titlebar::HEIGHT,
            lift,
        )
    }

    /// Whether the open menu is a dialog that dims the window behind it,
    /// rather than a popover beside what opened it.
    pub(crate) fn menu_dims(&self) -> bool {
        let Some(page) = self.menu.page else {
            return false;
        };
        spaceport_menu::chrome::dims(page, self.menu.session_modal(page))
    }
}

/// The font the popup chrome is set in.
pub(crate) fn text(ui: &FontConfig) -> Text {
    Text {
        font: crate::config::font_for(ui),
        line_height: ui.line_height(),
        size: ui.size,
    }
}

/// The window's own measurements the popup chrome lays out against.
pub(crate) fn metrics(viewport: Size<Pixels>) -> Metrics {
    Metrics {
        chrome: crate::titlebar::HEIGHT,
        viewport,
    }
}

/// Declares the footer menu's rows once: each [`Item`] with the English key
/// its `menu-{key}` debug selector is built from. The keys stay English
/// whatever language the labels are shown in.
macro_rules! items {
    ($($variant:ident => $selector:literal,)*) => {
        /// A row of the in-window app menu ([`Page::Menu`]).
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub(super) enum Item {
            $($variant,)*
        }

        impl Item {
            /// Every row, whether or not the window offers it now.
            #[cfg(test)]
            pub(super) const ALL: &[Self] = &[$(Self::$variant,)*];

            /// The row's element id and the tail of its debug selector.
            pub(super) fn selector(self) -> &'static str {
                match self {
                    $(Self::$variant => $selector,)*
                }
            }
        }
    };
}

items! {
    Settings => "settings",
    Shortcuts => "shortcuts",
    Themes => "themes",
    IncreaseFontSize => "increase font size", // i18n: data
    DecreaseFontSize => "decrease font size", // i18n: data
    ResetFontSize => "reset font size", // i18n: data
    CommandPalette => "command palette", // i18n: data
    Workspaces => "workspaces",
    ReloadGuiConfig => "reload GUI config", // i18n: data
    About => "about",
    ReloadDaemonConfig => "reload daemon config", // i18n: data
    UpdateReady => "update ready", // i18n: data
    WhatsNew => "what's new", // i18n: data
    Detach => "detach",
    Reconnect => "reconnect",
}

impl Item {
    /// What the menu shows for the row.
    pub(super) fn label(self) -> String {
        match self {
            Self::Settings => fl!("menu-chrome-settings"),
            Self::Shortcuts => fl!("menu-chrome-shortcuts"),
            Self::Themes => fl!("menu-chrome-themes"),
            Self::IncreaseFontSize => fl!("menu-chrome-increase-font-size"),
            Self::DecreaseFontSize => fl!("menu-chrome-decrease-font-size"),
            Self::ResetFontSize => fl!("menu-chrome-reset-font-size"),
            Self::CommandPalette => fl!("menu-chrome-command-palette"),
            Self::Workspaces => fl!("menu-chrome-workspaces"),
            Self::ReloadGuiConfig => fl!("menu-chrome-reload-gui-config"),
            Self::About => fl!("menu-chrome-about"),
            Self::ReloadDaemonConfig => fl!("menu-chrome-reload-daemon-config"),
            Self::UpdateReady => fl!("menu-chrome-update-ready"),
            Self::WhatsNew => fl!("menu-chrome-whats-new"),
            Self::Detach => fl!("menu-chrome-detach"),
            Self::Reconnect => fl!("menu-chrome-reconnect"),
        }
    }
}
