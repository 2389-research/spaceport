// ABOUTME: Routes raw shortcut events while retaining repeat and release semantics.
// ABOUTME: Keeps catalog, prefix, custom command, and resize routing in one window owner.
//! Herdr's prefix chords, as the TUI types them: the prefix arms the window,
//! and the next keystroke runs its chord's command or, when nothing is bound
//! to it, is swallowed. Typing the prefix twice sends it on to the focused
//! element as ordinary input, and Escape simply cancels. A chord typed in a
//! menu's text field closes the menu and runs, as it would from the terminal.
//!
//! The daemon's custom command shortcuts are matched here too, after the
//! keymap's own, and Herdr's resize mode claims every key until it ends.
//! A shifted key is also tried with the Shift GPUI dropped restored, after
//! every lookup has missed the key as GPUI named it.

use super::SheprdWindow;
use crate::controls::Command;
use gpui::{
    Context, Div, InteractiveElement as _, KeyDownEvent, KeyUpEvent, KeybindingKeystroke,
    Keystroke, Window, div,
};
use sheprd_client::protocol::ClientShellCommandAction;

mod shifted_keys;

pub(crate) use shifted_keys::KeyChars;
use shifted_keys::Typed;
pub(super) use shifted_keys::current_layout;

#[cfg(all(windows, feature = "integration-test"))]
pub(super) fn native_shortcuts_observed() -> bool {
    static OBSERVE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *OBSERVE.get_or_init(|| std::env::var_os("HERDR_TEST_SHORTCUT_OBSERVER").is_some())
}

#[cfg(all(windows, feature = "integration-test"))]
pub(super) fn native_shortcut_fences_observed() -> bool {
    static OBSERVE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    native_shortcuts_observed()
        && *OBSERVE.get_or_init(|| std::env::var_os("HERDR_TEST_SHORTCUT_FENCES").is_some())
}

/// What a keystroke is bound to here.
enum Bound {
    // keep-sorted start
    Command(Command),
    Custom(String, ClientShellCommandAction),
    Prefix,
    // keep-sorted end
}

/// The held tail of a shortcut stays with its original action until release.
#[derive(Clone, Copy, Debug)]
pub(crate) enum HeldShortcut {
    // keep-sorted start
    Consume(KeyOwner),
    PassThrough,
    Repeat(Command, KeyOwner),
    // keep-sorted end
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum KeyOwner {
    // keep-sorted start
    Direct,
    Modal,
    // keep-sorted end
}

impl HeldShortcut {
    fn yields_to_text(self) -> bool {
        matches!(
            self,
            Self::Consume(KeyOwner::Direct) | Self::Repeat(_, KeyOwner::Direct)
        )
    }
}

fn repeatable(command: Command) -> bool {
    matches!(
        command,
        Command::IncreaseFontSize
            | Command::DecreaseFontSize
            | Command::FindNext
            | Command::FindPrevious
            | Command::CycleWindows
            | Command::PreviousWorkspace
            | Command::NextWorkspace
            | Command::WorkspaceNumber(_)
            | Command::NextTab
            | Command::PreviousTab
            | Command::TabNumber(_)
            | Command::FocusLeft
            | Command::FocusRight
            | Command::FocusUp
            | Command::FocusDown
            | Command::NextPane
            | Command::PreviousPane
            | Command::LastPane
            | Command::PreviousAgent
            | Command::NextAgent
            | Command::AgentNumber(_)
            | Command::FocusNextRegion
            | Command::FocusPreviousRegion
            | Command::ResizeLeft
            | Command::ResizeRight
            | Command::ResizeUp
            | Command::ResizeDown
    )
}

impl SheprdWindow {
    /// The window's keyboard boundary, also usable without its terminal canvas.
    pub(crate) fn shortcut_scope(cx: &mut Context<Self>) -> Div {
        div()
            .key_context("SpaceportWindow")
            .capture_key_down(cx.listener(Self::shortcut_key_down))
            .capture_key_up(cx.listener(Self::shortcut_key_up))
    }

    /// Main-window catalog bindings are labels; this capture retains the raw repeat flag.
    pub(crate) fn shortcut_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !matches!(
            event.keystroke.key.as_str(),
            "shift" | "shiftleft" | "shiftright"
        ) {
            self.shift_taps.cancel();
        }
        let named = super::function_keys::named(&event.keystroke, self.key_press);
        let keystroke = named.as_ref().unwrap_or(&event.keystroke);
        let restored = shifted_keys::restore(keystroke, self.key_press, self.key_layout);
        let typed = Typed::new(keystroke, restored);
        #[cfg(all(windows, feature = "integration-test"))]
        {
            if native_shortcuts_observed() {
                let bound = typed.find_unheld(
                    |key| {
                        self.bound(key, false, event.prefer_character_input, |key| {
                            KeybindingKeystroke::new_with_mapper(
                                key,
                                false,
                                cx.keyboard_mapper().as_ref(),
                            )
                        })
                    },
                    |key| self.keymap().holds_directly(key),
                );
                let new_window = !self.prefix_armed
                    && matches!(&bound, Some(Bound::Command(Command::NewWindow)));
                let split_right = !self.prefix_armed
                    && matches!(&bound, Some(Bound::Command(Command::SplitRight)));
                let custom_command =
                    !self.prefix_armed && matches!(&bound, Some(Bound::Custom(..)));
                tracing::debug!(
                    is_held = event.is_held,
                    input_ready = self.input_ready(),
                    new_window,
                    split_right,
                    custom_command,
                    "Native shortcut input"
                );
            }
        }
        let key = typed.held_key();
        let previous = self.shortcut_keys.get(key).copied();
        if !event.is_held {
            self.shortcut_keys.remove(key);
        }
        // A consumed modal trigger must not end its mode on the held tail.
        if event.is_held
            && (self.prefix_armed
                || matches!(previous, Some(HeldShortcut::Consume(KeyOwner::Modal))))
        {
            cx.stop_propagation();
            window.prevent_default();
            return;
        }
        if self.resize_mode && self.resize_keystroke(&typed, window, cx) {
            if !self.resize_mode {
                self.shortcut_keys
                    .insert(key.to_owned(), HeldShortcut::Consume(KeyOwner::Modal));
            }
            return;
        }
        if event.is_held
            && let Some(held) = previous
        {
            if event.prefer_character_input && held.yields_to_text() {
                return;
            }
            match held {
                HeldShortcut::PassThrough => return,
                HeldShortcut::Repeat(command, _) => self.command(command, window, cx),
                HeldShortcut::Consume(_) => {}
            }
            cx.stop_propagation();
            window.prevent_default();
            return;
        }
        let held = self.shortcut_keystroke(
            &typed,
            event.is_held,
            event.prefer_character_input,
            window,
            cx,
        );
        if let Some(held) = held {
            self.shortcut_keys.insert(key.to_owned(), held);
            if !matches!(held, HeldShortcut::PassThrough) {
                cx.stop_propagation();
                window.prevent_default();
            }
        }
    }

    pub(crate) fn shortcut_key_up(
        &mut self,
        event: &KeyUpEvent,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        let named = super::function_keys::named(&event.keystroke, self.key_press);
        let keystroke = named.as_ref().unwrap_or(&event.keystroke);
        let restored = shifted_keys::restore(keystroke, self.key_press, self.key_layout);
        self.shortcut_keys
            .remove(Typed::new(keystroke, restored).held_key());
    }

    fn shortcut_keystroke(
        &mut self,
        typed: &Typed<'_>,
        is_held: bool,
        preferred_text: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<HeldShortcut> {
        // Every binding is tried on the key as GPUI named it before any is
        // tried on the restored key, so a binding on the symbol itself, such
        // as a custom command on `prefix+!`, still wins over one on
        // `prefix+shift+1`, and over a prefix on the restored key.
        let armed = self.prefix_armed;
        let map = |bound| {
            KeybindingKeystroke::new_with_mapper(bound, false, cx.keyboard_mapper().as_ref())
        };
        let bound = if armed {
            typed.find(|keystroke| self.bound(keystroke, true, preferred_text, map))
        } else {
            // Without the prefix, a key GPUI's bindings or a pane key hold
            // as GPUI named it is theirs; restoring its Shift must not take
            // it from them for the prefix or a custom command.
            let held = |keystroke: &Keystroke| self.keymap().holds_directly(keystroke);
            typed.find_unheld(
                |keystroke| self.bound(keystroke, false, preferred_text, map),
                held,
            )
        };
        if !armed {
            match bound {
                Some(Bound::Prefix) => {
                    if !is_held {
                        self.prefix_armed = true;
                    }
                    cx.notify();
                    return Some(HeldShortcut::Consume(KeyOwner::Modal));
                }
                Some(Bound::Custom(id, action)) => {
                    if !is_held {
                        self.invoke_custom_command(&id, action, cx);
                    }
                    return Some(HeldShortcut::Consume(KeyOwner::Direct));
                }
                Some(Bound::Command(command)) => {
                    if !is_held || repeatable(command) {
                        self.command(command, window, cx);
                    }
                    return Some(if repeatable(command) {
                        HeldShortcut::Repeat(command, KeyOwner::Direct)
                    } else {
                        HeldShortcut::Consume(KeyOwner::Direct)
                    });
                }
                None => {}
            }
            return None;
        }
        self.prefix_armed = false;
        cx.notify();
        let Some(bound) = bound else {
            return Some(HeldShortcut::Consume(KeyOwner::Modal));
        };
        // Typed twice, the prefix goes on as ordinary input.
        if matches!(bound, Bound::Prefix) {
            return Some(HeldShortcut::PassThrough);
        }
        // Commands wait while a menu page holds input, so the chord ends it
        // first. Run directly rather than through focus, which the menu held.
        if self.menu.page.is_some() {
            self.dismiss_menu(window, cx);
        }
        match bound {
            Bound::Command(command) => {
                self.command(command, window, cx);
                Some(if repeatable(command) {
                    HeldShortcut::Repeat(command, KeyOwner::Modal)
                } else {
                    HeldShortcut::Consume(KeyOwner::Modal)
                })
            }
            Bound::Custom(id, action) => {
                self.invoke_custom_command(&id, action, cx);
                Some(HeldShortcut::Consume(KeyOwner::Modal))
            }
            Bound::Prefix => Some(HeldShortcut::PassThrough),
        }
    }

    /// What `keystroke` is bound to: the prefix, then, after the prefix, the
    /// keymap's chord, then a custom command. Without the prefix, a custom
    /// command runs only while no menu page holds input.
    fn bound(
        &self,
        keystroke: &Keystroke,
        armed: bool,
        preferred_text: bool,
        map: impl Fn(Keystroke) -> KeybindingKeystroke,
    ) -> Option<Bound> {
        let keymap = self.keymap();
        if keymap.is_prefix(keystroke) {
            return Some(Bound::Prefix);
        }
        // Explicit prefix and resize modes own their input; direct shortcuts yield to IME/AltGr text.
        if !armed && preferred_text {
            return None;
        }
        if armed && let Some(command) = keymap.chord(keystroke) {
            if !keymap.owns_chord(keystroke)
                && let Some((id, action)) = self.custom_command(keystroke, true)
            {
                return Some(Bound::Custom(id, action));
            }
            return Some(Bound::Command(command));
        }
        if !armed && let Some(direct) = keymap.direct(keystroke, map) {
            let command = match direct {
                crate::keymap::Direct::Configured(command) => return Some(Bound::Command(command)),
                crate::keymap::Direct::Catalog(command) => command,
            };
            if self.menu.page.is_none()
                && let Some((id, action)) = self.custom_command(keystroke, false)
            {
                return Some(Bound::Custom(id, action));
            }
            return Some(Bound::Command(command));
        }
        if !armed && self.menu.page.is_some() {
            return None;
        }
        self.custom_command(keystroke, armed)
            .map(|(id, action)| Bound::Custom(id, action))
    }

    /// The selected daemon's custom command `keystroke` runs.
    fn custom_command(
        &self,
        keystroke: &Keystroke,
        prefixed: bool,
    ) -> Option<(String, ClientShellCommandAction)> {
        // As the palette and shortcut reference do, offer nothing while the
        // snapshot may be stale, so the key reaches the terminal instead.
        let snapshot = self
            .live
            .snapshot
            .as_ref()
            .filter(|_| self.live.status.is_connected())?;
        self.keymap()
            .custom_command(&snapshot.commands, keystroke, prefixed)
            .map(|command| (command.command_id.clone(), command.action))
    }

    /// Herdr's resize mode: h, j, k, l or the arrows resize the focused pane,
    /// Escape, Enter, or the mode's own shortcut ends it, and anything else
    /// is swallowed. Platform shortcuts such as quitting still work, and a
    /// menu, which takes keys of its own, ends the mode.
    fn resize_keystroke(
        &mut self,
        typed: &Typed<'_>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let keystroke = typed.keystroke();
        if self.menu.page.is_some() {
            self.resize_mode = false;
            cx.notify();
            return false;
        }
        if keystroke.modifiers.platform {
            return false;
        }
        cx.stop_propagation();
        window.prevent_default();
        let keymap = self.keymap();
        if matches!(keystroke.key.as_str(), "escape" | "enter")
            || typed.matches(|keystroke| {
                keymap.triggers(Command::ResizeMode, keystroke, true)
                    || keymap.triggers(Command::ResizeMode, keystroke, false)
            })
        {
            self.resize_mode = false;
            cx.notify();
            return true;
        }
        let modifiers = keystroke.modifiers;
        if modifiers.control || modifiers.alt || modifiers.function {
            return true;
        }
        let command = match keystroke.key.as_str() {
            "h" | "left" => Command::ResizeLeft,
            "j" | "down" => Command::ResizeDown,
            "k" | "up" => Command::ResizeUp,
            "l" | "right" => Command::ResizeRight,
            _ => return true,
        };
        self.command(command, window, cx);
        true
    }

    /// Leaving the window abandons a half-typed chord, as the TUI's prefix
    /// mode does not outlive its client, and ends resize mode.
    pub(super) fn disarm_prefix(&mut self) {
        self.prefix_armed = false;
        self.resize_mode = false;
        self.shortcut_keys.clear();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    mod platform_mapping;
    use crate::{
        keymap::{DaemonKeys, Keymap},
        state::ConnectionStatus,
    };
    use gpui::{Keystroke, TestAppContext, VisualTestContext};
    use sheprd_client::protocol::{ClientShellCommand, ClientShellCommandAction};
    use std::sync::Arc;

    /// Whether something handled the keystroke. Nothing binds `cmd-y` and
    /// the terminal ignores cmd keys, so only the prefix can claim it.
    fn press(keystroke: &str, cx: &mut VisualTestContext) -> bool {
        let handled = cx.update(|window, cx| {
            window.dispatch_keystroke(crate::keymap::parse_keystroke(keystroke).unwrap(), cx)
        });
        cx.run_until_parked();
        handled
    }

    #[gpui::test]
    fn the_prefix_runs_chords_and_swallows_other_keys(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
        let table: toml::Table = "prefix = ['cmd+j', 'cmd+u']\ntoggle_sidebar = 'prefix+cmd+z'"
            .parse()
            .unwrap();
        view.update(cx, |view, _| {
            view.config.keybindings = Keymap::with_overrides(
                &Default::default(),
                &Default::default(),
                &DaemonKeys::from_table(Some(&table)),
            )
            .unwrap();
        });
        let state = |cx: &mut VisualTestContext| {
            view.read_with(cx, |view, _| (view.prefix_armed, view.sidebar_visible))
        };
        cx.update(|window, cx| {
            window.focus(&view.read(cx).focus.clone(), cx);
            window.draw(cx).clear(cx);
        });
        assert!(!press("cmd-y", cx));
        assert!(
            !press("cmd-z", cx),
            "the chord's key alone is not a binding"
        );
        assert_eq!(state(cx), (false, true));

        assert!(press("cmd-j", cx));
        assert_eq!(state(cx), (true, true));
        assert!(press("cmd-z", cx));
        assert_eq!(state(cx), (false, false), "the chord ran");

        // Any configured prefix arms the same chords.
        assert!(press("cmd-u", cx));
        assert_eq!(state(cx), (true, false));
        assert!(press("cmd-z", cx));
        assert_eq!(state(cx), (false, true), "the second prefix ran the chord");
        assert!(press("cmd-j", cx));
        assert!(!press("cmd-u", cx), "another prefix passes through too");
        assert_eq!(state(cx), (false, true));
        assert!(press("cmd-u", cx));
        assert!(press("cmd-z", cx));
        assert_eq!(state(cx), (false, false));

        // An unbound key after the prefix goes nowhere, as in the TUI.
        assert!(press("cmd-j", cx));
        assert!(press("cmd-y", cx));
        assert_eq!(state(cx), (false, false));
        assert!(!press("cmd-y", cx), "only the next key is claimed");

        // The prefix twice passes the second one through.
        assert!(press("cmd-j", cx));
        assert!(!press("cmd-j", cx));
        assert_eq!(state(cx), (false, false));

        assert!(press("cmd-j", cx));
        assert!(press("escape", cx));
        assert_eq!(state(cx), (false, false));
        assert!(press("cmd-j", cx));
        assert!(press("cmd-z", cx));
        assert_eq!(state(cx), (false, true));

        // From a menu's focused text field, the chord closes the menu and runs.
        view.update_in(cx, |view, window, cx| view.open_keybinds(window, cx));
        cx.run_until_parked();
        let search_focused = |cx: &mut VisualTestContext| {
            cx.update(|window, cx| {
                let view = view.read(cx);
                view.menu.page.is_some()
                    && view
                        .menu
                        .keybinds_search
                        .as_ref()
                        .is_some_and(|search| search.read(cx).focus.is_focused(window))
            })
        };
        assert!(search_focused(cx));
        assert!(press("cmd-j", cx));
        assert!(search_focused(cx), "the prefix alone leaves the menu open");
        assert!(press("cmd-z", cx));
        assert_eq!(state(cx), (false, false));
        view.read_with(cx, |view, _| assert!(view.menu.page.is_none()));

        // An unbound key there is swallowed and leaves the menu open.
        view.update_in(cx, |view, window, cx| view.open_keybinds(window, cx));
        cx.run_until_parked();
        assert!(press("cmd-j", cx));
        assert!(press("cmd-y", cx));
        assert!(search_focused(cx));
    }

    /// A prefix that types text, such as `prefix = "@"` in the daemon's
    /// `[keys]`, arms however the keyboard reports it, and typing it twice
    /// sends it on.
    #[gpui::test]
    fn a_typing_prefix_arms_everywhere_and_twice_types_it(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
        let table: toml::Table = "prefix = '@'\ntoggle_sidebar = 'prefix+b'".parse().unwrap();
        view.update(cx, |view, _| {
            view.config.keybindings = Keymap::with_overrides(
                &Default::default(),
                &Default::default(),
                &DaemonKeys::from_table(Some(&table)),
            )
            .unwrap();
        });
        let state = |cx: &mut VisualTestContext| {
            view.read_with(cx, |view, _| (view.prefix_armed, view.sidebar_visible))
        };
        cx.update(|window, cx| {
            window.focus(&view.read(cx).focus.clone(), cx);
            window.draw(cx).clear(cx);
        });
        for typed in ["@", "shift-2->@"] {
            let visible = state(cx).1;
            assert!(press(typed, cx), "{typed}");
            assert_eq!(state(cx), (true, visible), "{typed}");
            assert!(press("b", cx), "{typed}");
            assert_eq!(state(cx), (false, !visible), "{typed} ran the chord");
        }

        // In a text field too, as in the TUI: twice types it once, and a
        // chord closes the menu and runs.
        let menu_open =
            |cx: &mut VisualTestContext| view.read_with(cx, |view, _| view.menu.page.is_some());
        let search = |cx: &mut VisualTestContext| {
            view.read_with(cx, |view, cx| {
                view.menu
                    .keybinds_search
                    .as_ref()
                    .map(|search| search.read(cx).text().to_owned())
            })
        };
        for typed in ["@", "shift-2->@"] {
            view.update_in(cx, |view, window, cx| view.open_keybinds(window, cx));
            cx.run_until_parked();
            assert_eq!(search(cx).as_deref(), Some(""), "{typed}");
            let visible = state(cx).1;
            assert!(press(typed, cx), "{typed}");
            assert_eq!(state(cx), (true, visible), "{typed}");
            assert_eq!(
                search(cx).as_deref(),
                Some(""),
                "{typed} arms, types nothing"
            );
            press(typed, cx);
            assert_eq!(state(cx), (false, visible), "{typed}");
            assert_eq!(search(cx).as_deref(), Some("@"), "{typed} twice types one");
            assert!(menu_open(cx), "{typed}");

            assert!(press(typed, cx), "{typed}");
            assert!(press("b", cx), "{typed}");
            assert_eq!(state(cx), (false, !visible), "{typed}");
            assert!(!menu_open(cx), "{typed} chord closed the menu");
        }
    }

    /// The shortcut reference and palette hide custom commands while the
    /// snapshot may be stale, so dispatch must not run them then either.
    #[gpui::test]
    fn custom_commands_run_only_while_connected(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
        let typed = Keystroke::parse("ctrl-alt-y").unwrap();
        view.update(cx, |view, _| {
            Arc::make_mut(view.live.snapshot.as_mut().unwrap()).commands =
                vec![ClientShellCommand {
                    command_id: "custom.y".into(),
                    description: None,
                    action: ClientShellCommandAction::Shell,
                    binding_label: "ctrl+alt+y".into(),
                    binding_labels: vec!["ctrl+alt+y".into()],
                }];
            for (status, runs) in [
                (ConnectionStatus::Connected, true),
                (ConnectionStatus::AwaitingSnapshot, true),
                (ConnectionStatus::Connecting, false),
                (ConnectionStatus::Disconnected, false),
                (ConnectionStatus::Detached, false),
            ] {
                view.live.status = status;
                assert_eq!(
                    view.custom_command(&typed, false).is_some(),
                    runs,
                    "{status:?}"
                );
            }
        });
    }
}
