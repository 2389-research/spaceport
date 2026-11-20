// ABOUTME: Routes window-owned commands through one dispatch table.
// ABOUTME: Keeps local actions separate from guarded daemon fallback.
//! The commands the window answers without the daemon, as one table: each
//! row names the commands it takes and the window method that runs them, and
//! the ones it does not take fall through to the daemon request.

use super::SheprdWindow;
use crate::controls::Command;
use gpui::{Context, Window};

/// What running one row of the table did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Run {
    /// It ran the command, and nothing else should.
    Done,
    /// It did the part that needs no daemon, and the request still follows.
    FallsThrough,
}

/// One row: the commands a handler takes, and the handler.
struct Row {
    takes: fn(Command) -> bool,
    run: fn(&mut SheprdWindow, Command, &mut Window, &mut Context<SheprdWindow>) -> Run,
}

/// Every command the window handles without the daemon, in the order it
/// tries them. A handler that does not take the command it was offered
/// returns [`Run::FallsThrough`], so the request it still needs is made.
const ROWS: &[Row] = &[
    Row {
        takes: |command| matches!(command, Command::CloseWorkspace),
        run: SheprdWindow::command_close_workspace,
    },
    Row {
        takes: |command| matches!(command, Command::OpenNotificationTarget),
        run: SheprdWindow::command_notification_target,
    },
    Row {
        takes: |command| matches!(command, Command::Logs),
        run: SheprdWindow::command_logs,
    },
    Row {
        takes: |command| matches!(command, Command::Messages),
        run: SheprdWindow::command_messages,
    },
    Row {
        takes: |command| matches!(command, Command::NewWindow),
        run: SheprdWindow::command_new_window,
    },
    Row {
        takes: |command| matches!(command, Command::ClosePane | Command::CloseTab),
        run: SheprdWindow::command_close,
    },
    Row {
        takes: |command| {
            matches!(
                command,
                Command::NextTab | Command::PreviousTab | Command::TabNumber(_)
            )
        },
        run: SheprdWindow::command_step_tab,
    },
    Row {
        takes: |command| matches!(command, Command::ReopenClosedTab),
        run: SheprdWindow::command_reopen_closed_tab,
    },
    Row {
        takes: |command| matches!(command, Command::SplitEditor),
        run: SheprdWindow::command_split_editor,
    },
    Row {
        takes: |command| matches!(command, Command::InstallBrowserSkill),
        run: SheprdWindow::command_install_browser_skill,
    },
    Row {
        takes: |command| matches!(command, Command::Palette | Command::WorkspacePicker),
        run: SheprdWindow::command_palette,
    },
    Row {
        takes: |command| matches!(command, Command::ResumePalette),
        run: SheprdWindow::command_resume_palette,
    },
    Row {
        takes: |command| matches!(command, Command::OpenWorktree),
        run: SheprdWindow::command_open_worktree,
    },
    Row {
        takes: |command| matches!(command, Command::WorktreeNotes),
        run: SheprdWindow::command_worktree_notes,
    },
    Row {
        takes: |command| matches!(command, Command::EditWorktreeNote),
        run: SheprdWindow::command_edit_worktree_note,
    },
    // Every interactive creation path ends here, so Herdr's name prompt
    // covers buttons, menus, shortcuts, and the palette alike.
    Row {
        takes: |command| matches!(command, Command::Tab | Command::Workspace),
        run: SheprdWindow::command_name_prompt,
    },
    Row {
        takes: |command| matches!(command, Command::Keybinds),
        run: SheprdWindow::command_keybinds,
    },
    Row {
        takes: |command| matches!(command, Command::Sessions),
        run: SheprdWindow::command_sessions,
    },
    Row {
        takes: |command| matches!(command, Command::Themes),
        run: SheprdWindow::command_themes,
    },
    Row {
        takes: |command| matches!(command, Command::Settings),
        run: SheprdWindow::command_settings,
    },
    Row {
        takes: |command| matches!(command, Command::CheckForUpdates),
        run: SheprdWindow::command_check_for_updates,
    },
    Row {
        takes: |command| matches!(command, Command::About),
        run: SheprdWindow::command_about,
    },
    Row {
        takes: |command| matches!(command, Command::Find),
        run: SheprdWindow::command_find,
    },
    // Repeats the pane's find, opening the bar when nothing is searched yet.
    Row {
        takes: |command| matches!(command, Command::FindNext | Command::FindPrevious),
        run: SheprdWindow::command_find_step,
    },
    Row {
        takes: |command| {
            matches!(
                command,
                Command::ToggleFullScreen | Command::CycleWindows | Command::CloseWindow
            )
        },
        run: SheprdWindow::command_window,
    },
    Row {
        takes: |command| matches!(command, Command::CopyMode),
        run: SheprdWindow::command_copy_mode,
    },
    Row {
        takes: |command| matches!(command, Command::EditScrollback),
        run: SheprdWindow::command_edit_scrollback,
    },
    Row {
        takes: |command| matches!(command, Command::ToggleSidebar),
        run: SheprdWindow::command_toggle_sidebar,
    },
    Row {
        takes: |command| matches!(command, Command::FocusNextRegion),
        run: SheprdWindow::command_focus_next_region,
    },
    Row {
        takes: |command| matches!(command, Command::FocusPreviousRegion),
        run: SheprdWindow::command_focus_previous_region,
    },
    // Window chrome only, so it runs while the daemon is still away.
    Row {
        takes: |command| matches!(command, Command::ToggleStatusBar),
        run: SheprdWindow::command_toggle_status_bar,
    },
    Row {
        takes: |command| matches!(command, Command::ToggleZenMode),
        run: SheprdWindow::command_toggle_zen,
    },
    Row {
        takes: |command| matches!(command, Command::DevicesOverview),
        run: SheprdWindow::command_devices_overview,
    },
    Row {
        takes: |command| matches!(command, Command::NewCoreTerminal),
        run: SheprdWindow::command_new_core_terminal,
    },
    // A herdr-core pane with the keyboard mirrors itself above.
    Row {
        takes: |command| matches!(command, Command::MirrorCorePane),
        run: SheprdWindow::command_mirror_core_pane,
    },
    Row {
        takes: |command| {
            matches!(
                command,
                Command::IncreaseFontSize | Command::DecreaseFontSize
            )
        },
        run: SheprdWindow::command_step_font_size,
    },
    Row {
        takes: |command| matches!(command, Command::ResetFontSize),
        run: SheprdWindow::command_reset_font_size,
    },
    Row {
        takes: |command| matches!(command, Command::ClearPane),
        run: SheprdWindow::command_clear_pane,
    },
    Row {
        takes: |command| matches!(command, Command::Detach),
        run: SheprdWindow::command_detach,
    },
    Row {
        takes: |command| matches!(command, Command::Reconnect),
        run: SheprdWindow::command_reconnect,
    },
    Row {
        takes: |command| matches!(command, Command::Quit),
        run: SheprdWindow::command_quit,
    },
    Row {
        takes: |command| matches!(command, Command::RenameTab),
        run: SheprdWindow::command_rename_tab,
    },
    Row {
        takes: |command| matches!(command, Command::RenamePane),
        run: SheprdWindow::command_rename_pane,
    },
    Row {
        takes: |command| matches!(command, Command::MovePane),
        run: SheprdWindow::command_move_pane,
    },
    Row {
        takes: |command| matches!(command, Command::RenameWorkspace),
        run: SheprdWindow::command_rename_workspace,
    },
    // As Herdr's binding does: the daemon rereads its file, and this
    // client its own, which holds the daemon's `[keys]` too.
    Row {
        takes: |command| matches!(command, Command::ReloadConfig),
        run: SheprdWindow::command_reload_config,
    },
    Row {
        takes: |command| matches!(command, Command::EqualizeSplits),
        run: SheprdWindow::command_equalize_splits,
    },
    Row {
        takes: |command| matches!(command, Command::ResizeMode),
        run: SheprdWindow::command_resize_mode,
    },
    Row {
        takes: |command| matches!(command, Command::LastPane),
        run: SheprdWindow::command_last_pane,
    },
    Row {
        takes: |command| matches!(command, Command::Back | Command::Forward),
        run: SheprdWindow::command_travel,
    },
    Row {
        takes: |command| {
            matches!(
                command,
                Command::PreviousWorkspace
                    | Command::NextWorkspace
                    | Command::WorkspaceNumber(_)
                    | Command::PreviousAgent
                    | Command::NextAgent
                    | Command::AgentNumber(_)
            )
        },
        run: SheprdWindow::command_step_sidebar,
    },
    // A browser tab moves without the daemon, so before its check.
    Row {
        takes: |command| matches!(command, Command::MoveTabPrevious | Command::MoveTabNext),
        run: SheprdWindow::command_move_tab,
    },
];

impl SheprdWindow {
    /// Runs the first row that takes `command`, or reports that the daemon
    /// request is all that is left.
    pub(super) fn dispatch(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Run {
        let Some(row) = ROWS.iter().find(|row| (row.takes)(command)) else {
            return Run::FallsThrough;
        };
        (row.run)(self, command, window, cx)
    }
}
