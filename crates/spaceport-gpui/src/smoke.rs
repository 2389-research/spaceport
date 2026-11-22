// ABOUTME: Registers the opt-in native fixtures and runs the one a run asks for.
// ABOUTME: Keeps each check in its own file behind one entry, and shared rules in `support`.
//! Native opt-in smoke driver: the fixture registry and the two entry points the
//! app calls, `start_live` for `--integration-test` and `start_sidebar` for
//! `--sidebar-test`.
//!
//! Each fixture lives in its own `smoke_*.rs` file, exposes one
//! `pub(crate) fn start`, and is registered once in the `fixtures!` table below,
//! so a new native check edits one file and adds one row. Files that a larger
//! fixture calls mid-sequence (`smoke_about`, `smoke_bidi`, `smoke_clipboard`,
//! `smoke_preferences`, `smoke_selection`) are checks, not fixtures, and keep a
//! descriptive name instead. Every fixture inherits the UI-thread rules in
//! `support`'s doc comment.
use crate::SpaceportWindow;
use anyhow::anyhow;
use gpui::{App, WindowHandle};

mod support;
pub(crate) use support::*;

#[cfg(test)]
#[path = "smoke/tests/fixtures.rs"]
mod fixture_tests;

#[path = "smoke_about.rs"]
mod about;

#[path = "smoke_bidi.rs"]
mod bidi;

#[cfg(target_os = "macos")]
#[path = "smoke_clipboard.rs"]
mod clipboard;

#[path = "smoke_live.rs"]
mod live;
pub(crate) use live::start as start_live;

#[cfg(target_os = "macos")]
#[path = "smoke_preferences.rs"]
mod preferences;

#[cfg(target_os = "macos")]
#[path = "smoke_selection.rs"]
mod selection;

/// Declares each registered fixture's file and registers its one entry: a row
/// is the module, the file that owns the check, and the environment variable
/// `--sidebar-test` selects it by, where `None` is the default fixture a run
/// without any variable runs.
macro_rules! fixtures {
    ($($(#[$meta:meta])* $module:ident: $file:literal, $variable:expr;)*) => {
        $(
            $(#[$meta])*
            #[path = $file]
            mod $module;
        )*

        /// The fixtures a `smoke_*` file owns, in table order; the first
        /// matching row runs.
        static FIXTURES: &[Fixture] = &[
            $(
                $(#[$meta])*
                Fixture { variable: $variable, start: $module::start },
            )*
        ];
    };
}

fixtures! {
    // keep-sorted start
    #[cfg(target_os = "macos")]
    hosts: "smoke_hosts.rs", Some("HERDR_TEST_SIDEBAR_ROUTING");
    #[cfg(target_os = "linux")]
    input: "smoke_input.rs", Some("HERDR_TEST_INPUT_SHUTDOWN");
    local_controls: "smoke_local_controls.rs", Some("HERDR_TEST_LOCAL_CONTROLS_ONLY");
    notifications: "smoke_notifications.rs", Some("HERDR_TEST_NOTIFICATIONS_ONLY");
    sidebar: "smoke_sidebar.rs", None;
    // keep-sorted end
}

/// Fixtures whose module already exists for other reasons: the chat preview and
/// start fixtures, and Linux's application menu bar.
static OWNED_ELSEWHERE: &[Fixture] = &[
    // keep-sorted start
    #[cfg(target_os = "linux")]
    Fixture {
        variable: Some("HERDR_TEST_APPLICATION_MENUS"),
        start: crate::menus::in_window::native::start,
    },
    Fixture {
        variable: Some("HERDR_TEST_CHAT_START_ERRORS"),
        start: crate::chat::native_start::start,
    },
    Fixture {
        variable: Some("HERDR_TEST_RICH_CHAT"),
        start: crate::chat::preview::start,
    },
    // keep-sorted end
];

/// One registered fixture: the variable that selects it, or `None` for the
/// default fixture, and the one entry in the file that owns the check.
struct Fixture {
    variable: Option<&'static str>,
    start: fn(WindowHandle<SpaceportWindow>, &mut App),
}

impl Fixture {
    /// Whether this run set the variable that selects this fixture.
    fn asked(&self) -> bool {
        self.variable
            .is_some_and(|name| std::env::var_os(name).is_some())
    }

    /// Whether this is the fixture a run without any variable runs.
    fn default_fixture(&self) -> bool {
        self.variable.is_none()
    }
}

/// Runs the daemon-free fixture this `--sidebar-test` run asked for.
pub fn start_sidebar(handle: WindowHandle<SpaceportWindow>, cx: &mut App) {
    let registered = || FIXTURES.iter().chain(OWNED_ELSEWHERE);
    // A variable selects its own fixture; a run without one runs the default.
    let selected = registered()
        .find(|fixture| fixture.asked())
        .or_else(|| registered().find(|fixture| fixture.default_fixture()));
    match selected {
        Some(fixture) => (fixture.start)(handle, cx),
        // The default row is part of the table, so this is a driver bug rather
        // than a fixture that failed.
        None => conclude(
            "SIDEBAR",
            "",
            Err(anyhow!("no native fixture is registered for this run")),
        ),
    }
}
