// ABOUTME: Opens and paints popup pages with shared window geometry.
// ABOUTME: Keeps popup input and focus isolated from terminal panes.
//! The popup shell: opening and dismissing it, keeping its input isolated from
//! the panes beneath, and painting the page the menu is currently on. Geometry
//! here is the same geometry used for hit testing and IME placement.

// i18n: migrated
mod keys;
mod render;

use super::Page;
use crate::SheprdWindow;
use crate::i18n::fl;
use gpui::{App, Context, Window};

impl SheprdWindow {
    pub(crate) fn workspace_shell_offset(&self) -> gpui::Pixels {
        self.menu.workspace_scroll.offset().y
    }

    pub(crate) fn open_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        self.shift_taps.cancel();
        if !self.cancel_theme_preview(cx) {
            return false;
        }
        self.menu.reset();
        self.apply_shared_theme(cx);
        self.menu.origin = Some(super::origin::Origin::capture(self));
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
        let core_close_focus = self.core_close_focus(cx);
        // A page that animates out keeps what it reads from the reset.
        if let Some(page) = self.menu.page {
            let kept = super::appear::Kept::take(self, window, cx);
            self.menu.appear.keep(page, kept);
        }
        self.menu.reset();
        self.apply_shared_theme(cx);
        window.focus(
            return_focus
                .as_ref()
                .or(core_close_focus.as_ref())
                .unwrap_or(&self.focus),
            cx,
        );
        cx.notify();
    }

    pub(crate) fn restore_menu_focus(&self, window: &mut Window, cx: &mut App) {
        if self.menu.page.is_none() && self.menu.focus.is_focused(window) {
            window.focus(&self.focus, cx);
        }
    }

    pub(crate) fn menu_target_current(&self) -> bool {
        self.menu
            .origin
            .as_ref()
            .is_some_and(|origin| origin.legacy_current(self))
    }

    pub(crate) fn menu_current(&self) -> bool {
        self.menu
            .origin
            .as_ref()
            .is_some_and(|origin| origin.current(self))
    }

    /// Asks the selected daemon to reread its config file.
    pub(crate) fn reload_daemon_config(&mut self) {
        let Some(endpoint) = self.endpoints.get_mut(self.selected_endpoint) else {
            return;
        };
        let error = endpoint.reload_config().err().map(|error| {
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
        let endpoint = self.endpoints.get(self.selected_endpoint);
        if endpoint.is_some() && self.live.status.is_connected() {
            items.push(Item::ReloadDaemonConfig);
        }
        items.extend(self.release_notes_item());
        if let Some(endpoint) = endpoint {
            items.push(if endpoint.connection.handle.is_some() {
                Item::Detach
            } else {
                Item::Reconnect
            });
        }
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
