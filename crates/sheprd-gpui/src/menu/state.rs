//! State one window's menu owns: which page is open, what it targets, and the
//! daemon answers still outstanding. Every modal action is fenced by the
//! selection epoch and connection generation it was started under, so a reply
//! from a replaced connection can never be applied to the current one.

// i18n: migrated
use super::{Page, WorkspaceMenuAction, WorkspaceTarget};
use crate::dialog_input::DialogInput;
use gpui::{App, Bounds, Entity, FocusHandle, Pixels, Point, ScrollHandle, Subscription};

/// What an open menu covers, as its last frame laid it out. Tests and native
/// checks read it to find the menu in the window.
#[cfg(any(windows, test, feature = "integration-test"))]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Cover {
    /// Not laid out yet.
    #[default]
    Unknown,
    /// A dialog dims the whole window.
    All,
    /// A popover or menu covers only its own panel.
    Panel(Bounds<Pixels>),
}

#[cfg(test)]
impl Cover {
    /// Whether a page drawn at `page` would hide this menu. Before the menu
    /// is laid out nothing is, so pages keep showing for that one frame.
    pub(crate) fn covers(self, page: Option<Bounds<Pixels>>) -> bool {
        match self {
            Self::Unknown => false,
            Self::All => true,
            Self::Panel(panel) => page.is_some_and(|page| page.intersects(&panel)),
        }
    }
}

pub(crate) struct MenuState {
    pub page: Option<Page>,
    pub(crate) application: Option<crate::menus::in_window::OpenMenu>,
    pub(crate) application_bar: crate::menus::in_window::Geometry,
    /// How the open popup, or the dimming a closed dialog left, moves.
    pub(crate) appear: super::appear::Appear,
    /// Written by the menu's layout, read by native interaction checks.
    #[cfg(any(windows, test, feature = "integration-test"))]
    pub(crate) cover: std::rc::Rc<std::cell::Cell<Cover>>,
    pub(super) device_setup: Option<super::devices::Setup>,
    pub(super) wsl_setup: Option<super::devices::WslSetup>,
    /// The saved distribution the removal confirmation names.
    pub(super) wsl_remove: Option<String>,
    pub(super) session_edit: Option<super::sessions::Edit>,
    /// The device whose sessions the picker shows under it, by endpoint id so
    /// a catalog change cannot unfold another one.
    pub(crate) unfolded_device: Option<String>,
    /// How tall the unfolded sessions were last frame, which their unfolding
    /// grows towards.
    pub(super) unfolded_height: std::rc::Rc<std::cell::Cell<Pixels>>,
    /// Where each line in the unfolded block last painted, so the keys can
    /// scroll to one session inside it rather than to the block.
    pub(super) unfolded_rows: std::rc::Rc<std::cell::RefCell<Vec<Bounds<Pixels>>>>,
    /// When a device last unfolded, so the list follows its sessions into
    /// view while they open.
    pub(super) unfolded_since: Option<std::time::Instant>,
    /// How open the unfolded sessions already were when they began to open,
    /// as when they reverse a fold that had not finished.
    pub(super) unfold_from: f32,
    /// The sessions still closing under the device that folded them.
    pub(super) folding: Option<super::devices::picker::Folding>,
    /// The address the open-link question is about.
    pub(super) open_link: Option<crate::terminal::ExternalUrl>,
    pub(super) devices_scroll: ScrollHandle,
    /// The picker line to scroll into view once the list has been laid out:
    /// on the frame it first appears, GPUI drops a scroll request unapplied.
    pub(super) devices_reveal: std::cell::Cell<Option<usize>>,
    pub(crate) usage_scroll: ScrollHandle,
    // Selection epoch and connection generation fence captured modal actions.
    pub(super) endpoint_target: (u64, u64),
    pub anchor: Point<Pixels>,
    /// Ignore repeated right presses until the opening gesture is released.
    pub(crate) opening_right_click: bool,
    pub focus: FocusHandle,
    pub(super) selected: Option<usize>,
    pub(super) workspace_selected: Option<WorkspaceMenuAction>,
    pub(super) target: Option<WorkspaceTarget>,
    pub input: Option<DialogInput>,
    pub(crate) error: Option<String>,
    /// The correlated `worktree.open` request, so the dialog can report its
    /// answer and follow the opened workspace.
    pub(super) opening: Option<String>,
    /// The name a new tab or workspace dialog proposed. Submitting it
    /// unchanged, or nothing, leaves naming to the daemon, as Herdr does.
    pub(super) suggested_name: Option<String>,
    pub(super) worktree_open: Option<super::worktree_open::Picker>,
    pub(super) keybinds_scroll: ScrollHandle,
    pub(crate) keybinds_search: Option<Entity<crate::search_input::SearchInput>>,
    pub(super) shortcut_editor: Option<super::shortcut_editor::Editor>,
    pub(super) _keybinds_subscription: Option<Subscription>,
    pub(crate) themes: Option<crate::theme_picker::ThemePicker>,
    pub(crate) palette: Option<crate::palette::Palette>,
    /// Whether Go To shows its preview; Alt-P toggles it for the session.
    pub(crate) palette_preview: bool,
    /// What the last closed palette leaves for Resume Palette.
    pub(crate) palette_resume: Option<crate::palette::Resume>,
    pub(crate) close: Option<crate::close_modal::CloseConfirmation>,
    pub(crate) tab: Option<crate::tab_menu::TabMenu>,
    pub(crate) group: Option<crate::group_menu::GroupMenu>,
    pub(crate) host: Option<super::devices::HostMenu>,
    pub(crate) pane: Option<crate::pane_menu::PaneMenu>,
    /// Saved devices whose removal is running, by endpoint ID. Kept after
    /// success until the catalog drops the device, so its header pulses
    /// until it disappears; the confirmation closes as soon as it starts.
    pub(crate) removing_devices: std::collections::HashSet<String>,
    pub(crate) version_notice: Option<super::VersionNotice>,
}

/// What a submitted dialog did, which decides whether it stays open.
pub(super) enum Submission {
    /// The request is queued and the dialog has nothing left to wait for.
    Queued { focus_changed: bool },
    /// The dialog stays open for a daemon answer it still needs.
    Awaiting { focus_changed: bool },
}

impl Submission {
    pub(super) fn focus_changed(&self) -> bool {
        let (Submission::Queued { focus_changed } | Submission::Awaiting { focus_changed }) = self;
        *focus_changed
    }
}

impl MenuState {
    pub fn new(cx: &App) -> Self {
        Self {
            page: None,
            application: None,
            application_bar: Default::default(),
            appear: Default::default(),
            #[cfg(any(windows, test, feature = "integration-test"))]
            cover: Default::default(),
            device_setup: None,
            wsl_setup: None,
            wsl_remove: None,
            session_edit: None,
            unfolded_device: None,
            unfolded_height: Default::default(),
            unfolded_rows: Default::default(),
            unfolded_since: None,
            unfold_from: 0.,
            folding: None,
            open_link: None,
            devices_scroll: ScrollHandle::new(),
            devices_reveal: std::cell::Cell::new(None),
            usage_scroll: ScrollHandle::new(),
            endpoint_target: (0, 0),
            anchor: Point::default(),
            opening_right_click: false,
            focus: cx.focus_handle(),
            selected: None,
            workspace_selected: None,
            target: None,
            input: None,
            error: None,
            opening: None,
            suggested_name: None,
            worktree_open: None,
            keybinds_scroll: ScrollHandle::new(),
            keybinds_search: None,
            shortcut_editor: None,
            _keybinds_subscription: None,
            themes: None,
            palette: None,
            palette_resume: None,
            palette_preview: true,
            close: None,
            removing_devices: Default::default(),
            version_notice: None,
            tab: None,
            group: None,
            host: None,
            pane: None,
        }
    }

    pub fn reset(&mut self) {
        self.application = None;
        #[cfg(any(windows, test, feature = "integration-test"))]
        self.cover.set(Cover::Unknown);
        self.device_setup = None;
        self.wsl_setup = None;
        self.wsl_remove = None;
        self.session_edit = None;
        self.unfolded_device = None;
        self.folding = None;
        self.open_link = None;
        self.devices_scroll.set_offset(Point::default());
        self.devices_reveal.set(None);
        self.usage_scroll.set_offset(Point::default());
        self.opening_right_click = false;
        self.tab = None;
        self.group = None;
        self.host = None;
        self.pane = None;
        self.page = None;
        if let Some(palette) = self.palette.take() {
            self.palette_resume = Some(palette.resume());
        }
        self.shortcut_editor = None;
        self.selected = None;
        self.workspace_selected = None;
        self.target = None;
        self.input = None;
        self.error = None;
        self.version_notice = None;
        self.opening = None;
        self.suggested_name = None;
        self.worktree_open = None;
        self.close = None;
    }
}

#[cfg(test)]
mod cover_tests {
    use super::Cover;
    use gpui::{Bounds, point, px, size};

    #[test]
    fn only_pages_under_a_menu_step_aside() {
        let at = |x: f32, width: f32| Bounds::new(point(px(x), px(0.)), size(px(width), px(400.)));
        // A sessions popover on the left, a page on the right.
        let popover = Cover::Panel(at(40., 300.));
        assert!(!popover.covers(Some(at(900., 700.))));
        assert!(popover.covers(Some(at(200., 700.))));
        // A dimmed dialog covers every page; an unlaid menu none yet.
        assert!(Cover::All.covers(Some(at(900., 700.))));
        assert!(!Cover::Unknown.covers(Some(at(0., 1900.))));
        // A page not drawn yet is under nothing.
        assert!(!popover.covers(None));
    }
}
