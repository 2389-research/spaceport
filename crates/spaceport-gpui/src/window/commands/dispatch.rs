//! The commands the window answers without the daemon, as one table: each
//! row names the commands it takes and the window method that runs them, and
//! the ones it does not take fall through to the daemon request.

use super::SpaceportWindow;
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
    run: fn(&mut SpaceportWindow, Command, &mut Window, &mut Context<SpaceportWindow>) -> Run,
}

/// Every command the window handles without the daemon, in the order it
/// tries them. A handler that does not take the command it was offered
/// returns [`Run::FallsThrough`], so the request it still needs is made.
const ROWS: &[Row] = &[
    Row {
        takes: |command| matches!(command, Command::OpenNotificationTarget),
        run: SpaceportWindow::command_notification_target,
    },
    Row {
        takes: |command| matches!(command, Command::Logs),
        run: SpaceportWindow::command_logs,
    },
    Row {
        takes: |command| matches!(command, Command::NewWindow),
        run: SpaceportWindow::command_new_window,
    },
    Row {
        takes: |command| matches!(command, Command::ClosePane | Command::CloseTab),
        run: SpaceportWindow::command_close,
    },
    Row {
        takes: |command| {
            matches!(
                command,
                Command::NextTab | Command::PreviousTab | Command::TabNumber(_)
            )
        },
        run: SpaceportWindow::command_step_tab,
    },
    Row {
        takes: |command| matches!(command, Command::ReopenClosedTab),
        run: SpaceportWindow::command_reopen_closed_tab,
    },
    Row {
        takes: |command| matches!(command, Command::SplitEditor),
        run: SpaceportWindow::command_split_editor,
    },
    Row {
        takes: |command| matches!(command, Command::InstallBrowserSkill),
        run: SpaceportWindow::command_install_browser_skill,
    },
    Row {
        takes: |command| matches!(command, Command::Palette | Command::WorkspacePicker),
        run: SpaceportWindow::command_palette,
    },
    Row {
        takes: |command| matches!(command, Command::ResumePalette),
        run: SpaceportWindow::command_resume_palette,
    },
    Row {
        takes: |command| matches!(command, Command::OpenWorktree),
        run: SpaceportWindow::command_open_worktree,
    },
    Row {
        takes: |command| matches!(command, Command::WorktreeNotes),
        run: SpaceportWindow::command_worktree_notes,
    },
    Row {
        takes: |command| matches!(command, Command::EditWorktreeNote),
        run: SpaceportWindow::command_edit_worktree_note,
    },
    // Every interactive creation path ends here, so Herdr's name prompt
    // covers buttons, menus, shortcuts, and the palette alike.
    Row {
        takes: |command| matches!(command, Command::Tab | Command::Workspace),
        run: SpaceportWindow::command_name_prompt,
    },
    Row {
        takes: |command| matches!(command, Command::Keybinds),
        run: SpaceportWindow::command_keybinds,
    },
    Row {
        takes: |command| matches!(command, Command::Sessions),
        run: SpaceportWindow::command_sessions,
    },
    Row {
        takes: |command| matches!(command, Command::Themes),
        run: SpaceportWindow::command_themes,
    },
    Row {
        takes: |command| matches!(command, Command::Settings),
        run: SpaceportWindow::command_settings,
    },
    Row {
        takes: |command| matches!(command, Command::CheckForUpdates),
        run: SpaceportWindow::command_check_for_updates,
    },
    Row {
        takes: |command| matches!(command, Command::About),
        run: SpaceportWindow::command_about,
    },
    Row {
        takes: |command| matches!(command, Command::Find),
        run: SpaceportWindow::command_find,
    },
    // Repeats the pane's find, opening the bar when nothing is searched yet.
    Row {
        takes: |command| matches!(command, Command::FindNext | Command::FindPrevious),
        run: SpaceportWindow::command_find_step,
    },
    Row {
        takes: |command| {
            matches!(
                command,
                Command::ToggleFullScreen | Command::CycleWindows | Command::CloseWindow
            )
        },
        run: SpaceportWindow::command_window,
    },
    Row {
        takes: |command| matches!(command, Command::CopyMode),
        run: SpaceportWindow::command_copy_mode,
    },
    Row {
        takes: |command| matches!(command, Command::EditScrollback),
        run: SpaceportWindow::command_edit_scrollback,
    },
    Row {
        takes: |command| matches!(command, Command::ToggleSidebar),
        run: SpaceportWindow::command_toggle_sidebar,
    },
    Row {
        takes: |command| matches!(command, Command::FocusNextRegion),
        run: SpaceportWindow::command_focus_next_region,
    },
    Row {
        takes: |command| matches!(command, Command::FocusPreviousRegion),
        run: SpaceportWindow::command_focus_previous_region,
    },
    // Window chrome only, so it runs while the daemon is still away.
    Row {
        takes: |command| matches!(command, Command::ToggleStatusBar),
        run: SpaceportWindow::command_toggle_status_bar,
    },
    Row {
        takes: |command| matches!(command, Command::ToggleZenMode),
        run: SpaceportWindow::command_toggle_zen,
    },
    Row {
        takes: |command| matches!(command, Command::DevicesOverview),
        run: SpaceportWindow::command_devices_overview,
    },
    Row {
        takes: |command| matches!(command, Command::NewCoreTerminal),
        run: SpaceportWindow::command_new_core_terminal,
    },
    // A herdr-core pane with the keyboard mirrors itself above.
    Row {
        takes: |command| matches!(command, Command::MirrorCorePane),
        run: SpaceportWindow::command_mirror_core_pane,
    },
    Row {
        takes: |command| {
            matches!(
                command,
                Command::IncreaseFontSize | Command::DecreaseFontSize
            )
        },
        run: SpaceportWindow::command_step_font_size,
    },
    Row {
        takes: |command| matches!(command, Command::ResetFontSize),
        run: SpaceportWindow::command_reset_font_size,
    },
    Row {
        takes: |command| matches!(command, Command::ClearPane),
        run: SpaceportWindow::command_clear_pane,
    },
    Row {
        takes: |command| matches!(command, Command::Detach),
        run: SpaceportWindow::command_detach,
    },
    Row {
        takes: |command| matches!(command, Command::Reconnect),
        run: SpaceportWindow::command_reconnect,
    },
    Row {
        takes: |command| matches!(command, Command::Quit),
        run: SpaceportWindow::command_quit,
    },
    Row {
        takes: |command| matches!(command, Command::RenameTab),
        run: SpaceportWindow::command_rename_tab,
    },
    Row {
        takes: |command| matches!(command, Command::RenamePane),
        run: SpaceportWindow::command_rename_pane,
    },
    Row {
        takes: |command| matches!(command, Command::MovePane),
        run: SpaceportWindow::command_move_pane,
    },
    Row {
        takes: |command| matches!(command, Command::RenameWorkspace),
        run: SpaceportWindow::command_rename_workspace,
    },
    // As Herdr's binding does: the daemon rereads its file, and this
    // client its own, which holds the daemon's `[keys]` too.
    Row {
        takes: |command| matches!(command, Command::ReloadConfig),
        run: SpaceportWindow::command_reload_config,
    },
    Row {
        takes: |command| matches!(command, Command::EqualizeSplits),
        run: SpaceportWindow::command_equalize_splits,
    },
    Row {
        takes: |command| matches!(command, Command::ResizeMode),
        run: SpaceportWindow::command_resize_mode,
    },
    Row {
        takes: |command| matches!(command, Command::LastPane),
        run: SpaceportWindow::command_last_pane,
    },
    Row {
        takes: |command| matches!(command, Command::Back | Command::Forward),
        run: SpaceportWindow::command_travel,
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
        run: SpaceportWindow::command_step_sidebar,
    },
    // A browser tab moves without the daemon, so before its check.
    Row {
        takes: |command| matches!(command, Command::MoveTabPrevious | Command::MoveTabNext),
        run: SpaceportWindow::command_move_tab,
    },
];

impl SpaceportWindow {
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
