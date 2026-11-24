//! The words the pane and tab menus say. The moved models return an action's
//! variant; this module names it in the current language, as `docs/i18n.md`
//! asks of everything a crate below the app returns.

// i18n: migrated
use crate::i18n::fl;
use spaceport_menu::{pane, tab};

/// The row's text, which says what the pane's own state makes the row do.
pub(crate) fn pane_action(action: pane::Action, target: &pane::Target) -> String {
    use pane::Action;
    match action {
        Action::Rename => fl!("pane-menu-rename"),
        Action::SplitRight => fl!("pane-menu-split-right"),
        Action::SplitDown => fl!("pane-menu-split-down"),
        Action::Swap => fl!("pane-menu-swap"),
        Action::Zoom if target.zoomed => fl!("pane-menu-unzoom"),
        Action::Zoom => fl!("pane-menu-zoom"),
        Action::MoveToNewTab => fl!("pane-menu-move-to-new-tab"),
        Action::MoveToNewWorkspace => fl!("pane-menu-move-to-new-workspace"),
        Action::MoveTo => fl!("pane-menu-move-to"),
        Action::EditScrollback => fl!("pane-menu-edit-scrollback"),
        Action::RightClick if target.right_click_passthrough => fl!("pane-menu-right-click-menu"),
        Action::RightClick => fl!("pane-menu-right-click-pane"),
        Action::Processes => fl!("pane-menu-processes"),
        Action::Close => fl!("pane-menu-close"),
    }
}

/// The row's text.
pub(crate) fn tab_action(action: tab::Action) -> String {
    use tab::Action;
    match action {
        Action::NewTab => fl!("tab-menu-new-tab"),
        Action::Rename => fl!("tab-menu-rename"),
        Action::MoveToWorkspace => fl!("tab-menu-move-to-workspace"),
        Action::Mute => fl!("tab-menu-mute"),
        Action::Unmute => fl!("tab-menu-unmute"),
        Action::UnmuteWorkspace => fl!("tab-menu-unmute-workspace"),
        Action::UnmuteDevice => fl!("tab-menu-unmute-device"),
        Action::Close => fl!("tab-menu-close"),
    }
}
