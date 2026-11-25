//! GPUI actions the app registers and the keystrokes bound to them. Every
//! shortcut comes from the config's resolved `Keymap`, so the palette, the
//! menu bar, and the keymap cannot drift apart. The macOS Hide and Minimize
//! items are the exception: they are not catalog commands, and keep the
//! keystrokes macOS gives every app unless the keymap binds a command to one.

use crate::controls::{CTRL_SHORTCUTS, Command};
use crate::text_editing::Platform;
use gpui::{Action, App, KeyBinding, KeyDownEvent, Keystroke, Modifiers, actions};

actions!(
    herdr,
    [
        Quit,
        Hide,
        HideOthers,
        ShowAll,
        Minimize,
        ZoomWindow,
        PlaySound,
        RingBellPreview,
        ShowLogs,
        ToggleSlowAnimations
    ]
);

// Edit menu items. Each element registers only the ones it can perform, so
// GPUI disables the rest while that element has focus.
actions!(edit, [Cut, Copy, Paste, SelectAll]);

/// No element sets this context, so these bindings never match a keystroke.
/// They exist only so the Edit menu shows its standard shortcuts. The focused
/// element's key handler keeps owning the Edit keys, as it did before the menu.
const EDIT_MENU_LABELS: &str = "EditMenuLabels";

/// The keystroke an Edit menu item stands for. A menu click replays it
/// through the focused element's own key handler, so choosing the item and
/// pressing its shortcut cannot behave differently.
pub(crate) fn edit_key(key: &str) -> KeyDownEvent {
    KeyDownEvent {
        keystroke: Keystroke {
            modifiers: Modifiers {
                platform: !CTRL_SHORTCUTS,
                control: CTRL_SHORTCUTS,
                shift: CTRL_SHORTCUTS,
                ..Modifiers::default()
            },
            key: key.into(),
            key_char: None,
        },
        is_held: false,
        prefer_character_input: false,
    }
}

#[derive(Clone, PartialEq, serde::Deserialize, Action)]
#[action(no_json)]
pub(crate) struct RunCommand {
    pub(crate) command: Command,
}

/// Picks the sidebar layout, from View > Layout.
#[derive(Clone, PartialEq, serde::Deserialize, Action)]
#[action(no_json)]
pub(crate) struct SetLayout {
    pub(crate) mode: crate::config::LayoutMode,
}

/// Picks how chat views draw, from View > Agent Layout.
#[derive(Clone, PartialEq, serde::Deserialize, Action)]
#[action(no_json)]
pub(crate) struct SetChatLayout {
    pub(crate) mode: crate::config::ChatLayoutMode,
}

/// Picks how tab strips draw their tabs, from View > Tab Layout.
#[derive(Clone, PartialEq, serde::Deserialize, Action)]
#[action(no_json)]
pub(crate) struct SetTabLayout {
    pub(crate) mode: crate::config::TabLayoutMode,
}

#[derive(Clone, PartialEq, serde::Deserialize, Action)]
#[action(no_json)]
pub(crate) struct ShowToastPreview {
    pub(crate) kind: spaceport_client::protocol::SemanticNotificationKind,
}

/// QA > Pseudo-Locale: shows every translated message in `pseudo`, or the
/// `language` setting's locale again when it is already on.
#[derive(Clone, PartialEq, serde::Deserialize, Action)]
#[action(no_json)]
pub(crate) struct TogglePseudoLocale {
    pub(crate) pseudo: spaceport_i18n::Pseudo,
}

/// QA: posts an OS notification for the focused pane after a short delay.
#[derive(Clone, PartialEq, serde::Deserialize, Action)]
#[action(no_json)]
pub(crate) struct ShowSystemNotificationPreview;

#[cfg(any(target_os = "macos", windows, test))]
#[derive(Clone, PartialEq, serde::Deserialize, Action)]
#[action(no_json)]
pub(crate) struct SetBadgePreview {
    pub(crate) enabled: bool,
}

/// Binds the active window's server keymap when its device opted into one,
/// otherwise the keymap from the last validated config, or the catalog
/// defaults before any config has loaded.
pub(crate) fn bind_keys(cx: &mut App) {
    let keymap = cx
        .try_global::<crate::window::ActiveServerKeymap>()
        .and_then(|active| active.0.clone())
        .or_else(|| {
            cx.try_global::<crate::app::InitialAppearance>()
                .map(|appearance| appearance.keymap.clone())
        })
        .unwrap_or_default();
    // Shortcuts come from the user's config, so one GPUI refuses is skipped
    // and logged rather than taking the app down.
    let bindings: Vec<_> = keymap
        .bindings()
        .filter_map(|(command, keystroke)| {
            let action: Box<dyn Action> = if command == Command::Quit {
                Box::new(Quit)
            } else {
                Box::new(RunCommand { command })
            };
            KeyBinding::load(keystroke, action, None, false, None, cx)
                .inspect_err(|error| tracing::warn!(%error, keystroke, "skipped a shortcut"))
                .ok()
        })
        .collect();
    cx.bind_keys(bindings);
    cx.bind_keys(crate::log_window::key_bindings());
    cx.bind_keys(crate::settings_window::key_bindings());
    let [cut, copy, paste, select_all] = EDIT_KEYS;
    cx.bind_keys([
        KeyBinding::new(cut, Cut, Some(EDIT_MENU_LABELS)),
        KeyBinding::new(copy, Copy, Some(EDIT_MENU_LABELS)),
        KeyBinding::new(paste, Paste, Some(EDIT_MENU_LABELS)),
        KeyBinding::new(select_all, SelectAll, Some(EDIT_MENU_LABELS)),
    ]);
    // Bound last, where they outrank bindings at the same context depth. One
    // the keymap binds a command to is left out instead, so its menu item
    // shows no shortcut it does not have.
    #[cfg(target_os = "macos")]
    {
        let [hide, hide_others, minimize] = SYSTEM_KEYS;
        let free: Vec<_> = free_system_keys(&keymap).collect();
        cx.bind_keys(
            [
                KeyBinding::new(hide, Hide, None),
                KeyBinding::new(hide_others, HideOthers, None),
                KeyBinding::new(minimize, Minimize, None),
            ]
            .into_iter()
            .zip(SYSTEM_KEYS)
            .filter(|(_, keystroke)| free.contains(keystroke))
            .map(|(binding, _)| binding),
        );
    }
}

/// Fields also accept plain Ctrl on Linux and Windows, where there is no
/// terminal program to receive it. Keep the terminal's Ctrl-Shift aliases
/// working in fields so Edit menu clicks and keyboard shortcuts share the
/// same path.
pub(crate) fn edit_modifiers(modifiers: Modifiers) -> bool {
    Platform::CURRENT.edit_modifiers(modifiers)
}

// The keystrokes the app itself holds — the Edit menu's four and macOS's
// system keys — spelled by `spaceport-keymap`, where a daemon custom command
// is kept off them. `bind_keys` above binds them to this app's actions.
#[cfg(any(target_os = "macos", test))]
pub use spaceport_keymap::free_system_keys;
#[cfg(test)]
pub use spaceport_keymap::reserved_keystrokes;
pub use spaceport_keymap::{EDIT_KEYS, SYSTEM_KEYS};

/// Replaces every binding after a config reload. The menu bar reads its
/// shortcut labels from the keymap when it is installed, so it is rebuilt too.
pub(crate) fn rebind_keys(cx: &mut App) {
    cx.clear_key_bindings();
    bind_keys(cx);
    crate::menus::install(cx);
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
