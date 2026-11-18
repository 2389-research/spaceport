// ABOUTME: Captures the visible data a closing popup needs after its owners reset.
// ABOUTME: Keeps departure paint separate from active readers and input ownership.
//! What a closing popup draws from while it goes. Closing resets the menu's
//! state in the same frame; a page that animates out has the fields it reads
//! taken out first, and drawn again from here until it is gone. Taking them
//! out ends their effect before the leaving animation finishes.

use super::super::Page;

/// The bounded warning rows and scroll location captured before reset.
pub(in crate::menu) struct MessagesPaint {
    pub(in crate::menu) entries: Vec<(crate::notice::Notice, String)>,
    pub(in crate::menu) offset: gpui::Point<gpui::Pixels>,
}

/// The fields a leaving page reads that closing clears.
#[derive(Default)]
pub(in crate::menu) struct Kept {
    pub(in crate::menu) shell_offset: Option<gpui::Pixels>,
    pub(in crate::menu) palette: Option<crate::palette::Paint>,
    pub(in crate::menu) footer: Option<(Vec<super::super::chrome::Item>, Option<usize>)>,
    pub(in crate::menu) messages: Option<MessagesPaint>,
    pub(in crate::menu) static_dialog: Option<super::super::static_paint::StaticPaint>,
    pub(in crate::menu) close: Option<crate::close_modal::Paint>,
    pub(in crate::menu) themes: Option<crate::theme_picker::Paint>,
    pub(in crate::menu) keybinds: Option<super::super::settings::KeybindsPaint>,
    pub(in crate::menu) group: Option<crate::group_menu::Paint>,
    pub(in crate::menu) device_theme: Option<super::super::devices::DeviceThemePaint>,
    pub(in crate::menu) host: Option<super::super::devices::HostPaint>,
    pub(in crate::menu) tab: Option<crate::tab_menu::Paint>,
    pub(in crate::menu) application: Option<crate::menus::in_window::Paint>,
    pub(in crate::menu) workspace_popover: Option<super::super::workspace::PopoverPaint>,
    pub(in crate::menu) workspace_dialog: Option<super::super::workspace::DialogPaint>,
    pub(in crate::menu) usage: Option<crate::usage::Paint>,
    pub(in crate::menu) device_setup: Option<super::super::devices::DeviceSetupPaint>,
    pub(in crate::menu) pane: Option<crate::pane_menu::Paint>,
    pub(in crate::menu) core_workspace: Option<crate::core_terminals::WorkspacePaint>,
    pub(in crate::menu) devices: Option<super::super::devices::DevicesPaint>,
}

impl Kept {
    /// Takes what the open page reads out of `menu`, before its reset.
    pub(in crate::menu) fn take(
        view: &mut crate::SheprdWindow,
        window: &gpui::Window,
        cx: &gpui::App,
    ) -> Self {
        let workspace_dialog = if matches!(view.menu.page, Some(Page::Dialog(_))) {
            view.workspace_dialog_paint(cx)
        } else {
            None
        };
        let core_workspace = view.core_workspace_paint(cx);
        Self {
            shell_offset: view.menu_shell_offset(),
            palette: view.palette_paint(window.viewport_size(), cx),
            footer: (view.menu.page == Some(Page::Menu))
                .then(|| (view.menu_items(), view.menu.selected)),
            messages: (view.menu.page == Some(Page::Messages)).then(|| MessagesPaint {
                entries: view.messages(),
                offset: view.menu.messages_scroll.offset(),
            }),
            static_dialog: view
                .menu
                .page
                .and_then(|page| view.static_dialog_paint(page, cx)),
            close: view.close_confirmation_paint(),
            themes: view
                .menu
                .page
                .filter(|page| *page == Page::Themes)
                .and_then(|_| view.theme_picker_paint(cx)),
            keybinds: (view.menu.page == Some(Page::Keybinds)).then(|| view.keybinds_paint(cx)),
            group: view
                .menu
                .page
                .filter(|page| *page == Page::Group)
                .and_then(|_| view.group_menu_paint(cx)),
            device_theme: view
                .menu
                .page
                .filter(|page| *page == Page::DeviceTheme)
                .and_then(|_| view.device_theme_paint(cx)),
            host: view
                .menu
                .page
                .filter(|page| {
                    matches!(
                        page,
                        Page::Host
                            | Page::RenameDevice
                            | Page::ForwardPort
                            | Page::DeviceTags
                            | Page::RemoveDevice
                    )
                })
                .and_then(|_| view.host_paint(cx)),
            tab: view
                .menu
                .page
                .filter(|page| matches!(page, Page::Tab | Page::RenameTab | Page::MoveTab))
                .and_then(|_| view.tab_menu_paint(cx)),
            application: view.application_menu_paint(window, cx),
            workspace_popover: (view.menu.page == Some(Page::Workspace))
                .then(|| view.workspace_popover_paint())
                .flatten(),
            workspace_dialog,
            core_workspace,
            devices: view.devices_paint(cx),
            pane: view
                .menu
                .page
                .filter(|page| page.pane_menu())
                .and_then(|_| view.pane_menu_paint(cx)),
            device_setup: view
                .menu
                .page
                .filter(|page| matches!(page, Page::AddDevice | Page::AddWsl | Page::RemoveWsl))
                .and_then(|_| view.device_setup_paint(cx)),
            usage: view.menu.page.and_then(|page| match page {
                Page::Usage(provider) => Some(view.usage_paint(provider)),
                _ => None,
            }),
        }
    }

    /// Every page must explicitly identify its passive capture before departure can draw it.
    pub(in crate::menu) fn holds(&self, page: Page) -> bool {
        match page {
            Page::About
            | Page::AgentSkill
            | Page::OpenLink
            | Page::Update
            | Page::VersionMismatch => self.static_dialog.is_some(),
            Page::AddDevice | Page::AddWsl | Page::RemoveWsl => self.device_setup.is_some(),
            Page::Application => self.application.is_some(),
            Page::ConfirmClose => self.close.is_some(),
            Page::CoreWorkspaceActions
            | Page::CoreWorkspaceCreate
            | Page::CoreWorkspaceDelete
            | Page::CoreWorkspaceRename => self.core_workspace.is_some(),
            Page::Devices => self.devices.is_some(),
            Page::DeviceTags
            | Page::ForwardPort
            | Page::Host
            | Page::RemoveDevice
            | Page::RenameDevice => self.host.is_some(),
            Page::DeviceTheme => self.device_theme.is_some(),
            Page::Dialog(_) => self.workspace_dialog.is_some(),
            Page::Group => self.group.is_some(),
            Page::Keybinds => self.keybinds.is_some(),
            Page::KillProcesses
            | Page::MovePane
            | Page::Pane
            | Page::PaneProcesses
            | Page::RenamePane => self.pane.is_some(),
            Page::Menu => self.footer.is_some(),
            Page::Messages => self.messages.is_some(),
            Page::MoveTab | Page::RenameTab | Page::Tab => self.tab.is_some(),
            Page::Palette => self.palette.is_some(),
            Page::Themes => self.themes.is_some(),
            Page::Usage(_) => self.usage.is_some(),
            Page::Workspace => self.workspace_popover.is_some(),
        }
    }
}
