// objc 0.2's selectors expand a legacy cargo-clippy cfg in the native test adapter.
#![cfg_attr(feature = "integration-test", allow(unexpected_cfgs))]
// Tests start real processes; production code builds them with `new_command`.
#![cfg_attr(test, allow(clippy::disallowed_methods))]

mod about;
mod accessibility;
mod actions;
mod agent_skill;
mod ago;
mod app;
#[cfg(any(target_os = "macos", windows, test))]
mod app_badge;
mod app_icon;
mod automations;
mod avatars;
mod backdrop;
mod bell;
mod browser;
mod bundled;
mod caffeine;
mod caption;
mod caret;
mod chat;
mod cli;
mod close_modal;
mod config;
mod config_diagnostic;
mod connection;
mod constants;
mod contrast;
mod control;
mod copy_mode;
mod core_terminals;
mod daemon;
mod device_tags;
mod devices_overview;
mod dialog_input;
mod direction;
mod endpoint;
mod error;
mod find;
mod font_picker;
mod fonts;
mod group_menu;
mod group_terminals;
mod herdr_settings;
mod i18n;
mod icon_font_notice;
mod icons;
mod input;
mod integrations;
#[cfg(windows)]
mod jump_list;
mod keymap;
#[cfg(windows)]
mod launch_at_login;
mod lenient;
mod link;
mod links;
mod listening_ports;
mod log_targets;
mod log_window;
mod mark_layer;
mod menu;
mod menus;
mod motion;
mod multiline_input;
mod navigation;
mod network_quality;
mod notification_mutes;
mod notifications;
mod osc52;
mod overview;
mod palette;
mod pane_menu;
mod pane_move;
mod panel_resize;
mod port_forward;
mod preferences;
mod presentation;
mod processes;
mod progress;
mod release_notes;
mod reorder;
mod restart_manager;
mod scrollback;
mod search_input;
mod sessions;
mod settings_panel;
mod settings_window;
#[cfg(unix)]
mod setup_relay;
mod sidebar;
mod smooth_scroll;
mod sound;
mod state;
mod state_file;
mod storage_warning;
mod system_load;
mod tab_layout;
mod tab_menu;
mod taskbar;
mod teleport;
mod terminal;
mod terminal_painter;
mod text_editing;
mod theme_picker;
mod titlebar;
mod toggles;
mod truncate;
mod update_panel;
mod usage;
mod window;
mod window_state;
mod words;
mod worktree;
mod worktree_banner;
mod worktree_identity;
mod worktree_notes;

#[cfg(feature = "integration-test")]
mod performance;
#[cfg(feature = "integration-test")]
mod smoke;
#[cfg(all(test, unix))]
#[path = "../../test-support/executable.rs"]
mod test_executable;
#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
#[allow(unsafe_code)]
#[path = "../../test-support/process.rs"]
mod test_process;

pub use error::{Error, Result};

// Its own crate so it builds and is tested without GPUI; the alias keeps the
// app's `crate::updater` paths.
use sheprd_updater as updater;
// The command catalog and its default shortcuts, without GPUI.
use sheprd_commands as controls;
// The log file and crash reports, without GPUI.
use sheprd_diagnostics::{self as diagnostics, crash};
// Git status, without GPUI.
// Shared with herdr-client's own SSH and probe children.
use sheprd_client::{local_path, login_env};
// Host scripts and Teleport's core.

pub(crate) use {
    actions::{
        CheckForUpdates, Hide, HideOthers, Minimize, PlaySound, Quit, RunCommand, ShowAll,
        ShowLogs, ShowUpdatePreview, ZoomWindow, bind_keys,
    },
    app::open_additional_window,
    constants::{APP_VERSION, RELEASE_BUILD, TAB_HEIGHT, TAB_WIDTH, WINDOW_TITLE},
    menus::menus,
    navigation::{NavigationTarget, OwnedNavigationTarget},
    window::SheprdWindow,
};

// Re-exported at the root because they are referenced crate-wide; the domain
// modules below are where they are defined and changed.
#[cfg(feature = "integration-test")]
pub(crate) use app::open_window;
// Only the macOS smoke checks drive the connection status from the root.
#[cfg(all(feature = "integration-test", target_os = "macos"))]
pub(crate) use state::ConnectionStatus;

pub(crate) use {controls::Command, state::LiveState, terminal::WheelAccumulator};

fn main() -> std::process::ExitCode {
    let exit = app::run();
    if exit != std::process::ExitCode::SUCCESS {
        return exit;
    }
    #[cfg(feature = "integration-test")]
    return std::process::ExitCode::from(
        smoke::EXIT_CODE.load(std::sync::atomic::Ordering::SeqCst),
    );
    #[cfg(not(feature = "integration-test"))]
    std::process::ExitCode::SUCCESS
}
