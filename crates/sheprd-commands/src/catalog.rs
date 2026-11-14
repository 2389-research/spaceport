//! The one declaration of every catalog command. Each row of the table at
//! the end of this file names a command once, and `catalog!` writes the
//! [`Command`] enum, [`COMMANDS`], and the macOS and Linux default shortcuts
//! from it, so a command cannot have a variant and no row, or the reverse.

use crate::CommandInfo;

/// A plain row is one command:
///
/// ```text
/// Variant, "keybinding_name", "message-id", mac: ["key", ..], linux: ["key", ..];
/// ```
///
/// A numbered row is one `Variant(u8)` and a command for each number listed.
/// The number ends the keybinding name and each shortcut:
///
/// ```text
/// Variant(1 2 3), "keybinding_name_", "message-id", mac: ["key-"], linux: ["key-"];
/// ```
///
/// Doc comments above a row document its variant. The `@` rules are the
/// macro's own steps: `@rows` reads one row at a time into the variants and
/// the commands read so far, then writes the items once no row is left.
macro_rules! catalog {
    (@rows [$($variants:tt)*] [$($commands:tt)*]
        $(#[$meta:meta])*
        $variant:ident($($number:literal)+), $name:literal, $message:literal,
        mac: $mac:tt, linux: $linux:tt;
        $($rest:tt)*
    ) => {
        catalog! {
            @rows
            [$($variants)* $(#[$meta])* $variant(u8),]
            [$($commands)* $({
                [Command::$variant($number)]
                concat!($name, $number), $message,
                catalog!(@numbered $mac $number), catalog!(@numbered $linux $number)
            })+]
            $($rest)*
        }
    };
    (@rows [$($variants:tt)*] [$($commands:tt)*]
        $(#[$meta:meta])*
        $variant:ident, $name:literal, $message:literal,
        mac: [$($mac:literal),*], linux: [$($linux:literal),*];
        $($rest:tt)*
    ) => {
        catalog! {
            @rows
            [$($variants)* $(#[$meta])* $variant,]
            [$($commands)* {
                [Command::$variant] $name, $message, &[$($mac),*], &[$($linux),*]
            }]
            $($rest)*
        }
    };
    (@rows [$($variants:tt)*] [$({
        [$($command:tt)*] $name:expr, $message:expr, $mac:expr, $linux:expr
    })*]) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Deserialize)]
        pub enum Command {
            $($variants)*
        }

        /// In the order the palette and the keybindings window list them.
        pub const COMMANDS: &[CommandInfo] = &[
            $(CommandInfo {
                command: $($command)*,
                name: $name,
                message: $message,
            },)*
        ];

        pub(crate) mod macos {
            use super::Command;

            pub(crate) fn defaults(command: Command) -> &'static [&'static str] {
                match command {
                    $($($command)* => $mac,)*
                    // A number no row lists.
                    _ => &[],
                }
            }
        }

        pub(crate) mod linux {
            use super::Command;

            /// Linux reserves Super for the desktop and Ctrl for terminal programs.
            /// Explicit defaults keep shifted Mac actions from collapsing onto the
            /// same Ctrl-Shift key, and leave Ctrl-Shift-C/V available for the clipboard.
            pub(crate) fn defaults(command: Command) -> &'static [&'static str] {
                match command {
                    $($($command)* => $linux,)*
                    // A number no row lists.
                    _ => &[],
                }
            }
        }
    };
    (@numbered [$($prefix:literal),*] $number:literal) => {
        &[$(concat!($prefix, $number)),*]
    };
    ($($rows:tt)*) => {
        catalog! { @rows [] [] $($rows)* }
    };
}

// The order is the palette's and the keybindings window's, so it is not
// sorted. Each `"keybinding_name"` is a key of the config file's
// `[keybindings]`: renaming one breaks existing configs.
catalog! {
    OpenNotificationTarget, "open_notification_target", "commands-open-notification-target",
        mac: ["cmd-alt-n"], linux: ["ctrl-alt-shift-u"];
    Logs, "logs", "commands-logs", mac: [], linux: [];
    NewWindow, "new_window", "commands-new-window", mac: ["cmd-alt-shift-n"], linux: ["ctrl-alt-n"];
    Workspace, "new_workspace", "commands-new-workspace",
        mac: ["cmd-shift-n"], linux: ["ctrl-alt-shift-n"];
    /// The focused repository's "Open worktree..." dialog.
    OpenWorktree, "open_worktree", "commands-open-worktree", mac: [], linux: [];
    /// Go To, listing only the checkouts that have a note.
    WorktreeNotes, "worktree_notes", "commands-worktree-notes", mac: [], linux: [];
    /// The focused checkout's note.
    EditWorktreeNote, "edit_worktree_note", "commands-edit-worktree-note", mac: [], linux: [];
    PreviousWorkspace, "previous_workspace", "commands-previous-workspace", mac: [], linux: [];
    NextWorkspace, "next_workspace", "commands-next-workspace", mac: [], linux: [];
    WorkspaceNumber(1 2 3 4 5 6 7 8 9), "focus_workspace_", "commands-focus-workspace",
        mac: [], linux: [];
    RenameWorkspace, "rename_workspace", "commands-rename-workspace", mac: [], linux: [];
    Tab, "new_tab", "commands-new-tab", mac: ["cmd-t"], linux: ["ctrl-shift-t"];
    SplitRight, "split_right", "commands-split-right", mac: ["cmd-d"], linux: ["ctrl-shift-d"];
    SplitDown, "split_down", "commands-split-down",
        mac: ["cmd-shift-d"], linux: ["ctrl-alt-shift-d"];
    NextTab, "next_tab", "commands-next-tab", mac: ["cmd-shift-]"], linux: ["ctrl-pagedown"];
    PreviousTab, "previous_tab", "commands-previous-tab",
        mac: ["cmd-shift-["], linux: ["ctrl-pageup"];
    MoveTabPrevious, "move_tab_previous", "commands-move-tab-previous", mac: [], linux: [];
    MoveTabNext, "move_tab_next", "commands-move-tab-next", mac: [], linux: [];
    RenameTab, "rename_tab", "commands-rename-tab", mac: [], linux: [];
    FocusLeft, "focus_left", "commands-focus-left",
        mac: ["cmd-alt-left"], linux: ["ctrl-alt-shift-left"];
    FocusRight, "focus_right", "commands-focus-right",
        mac: ["cmd-alt-right"], linux: ["ctrl-alt-shift-right"];
    FocusUp, "focus_up", "commands-focus-up", mac: ["cmd-alt-up"], linux: ["ctrl-alt-shift-up"];
    FocusDown, "focus_down", "commands-focus-down",
        mac: ["cmd-alt-down"], linux: ["ctrl-alt-shift-down"];
    NextPane, "next_pane", "commands-next-pane", mac: ["cmd-alt-]"], linux: ["ctrl-alt-shift-]"];
    PreviousPane, "previous_pane", "commands-previous-pane",
        mac: ["cmd-alt-["], linux: ["ctrl-alt-shift-["];
    LastPane, "last_pane", "commands-last-pane", mac: [], linux: [];
    /// Back through the panes this endpoint has focused.
    Back, "back", "commands-back", mac: ["cmd-["], linux: ["ctrl-shift-["];
    Forward, "forward", "commands-forward", mac: ["cmd-]"], linux: ["ctrl-shift-]"];
    SwapLeft, "swap_pane_left", "commands-swap-pane-left", mac: [], linux: [];
    SwapRight, "swap_pane_right", "commands-swap-pane-right", mac: [], linux: [];
    SwapUp, "swap_pane_up", "commands-swap-pane-up", mac: [], linux: [];
    SwapDown, "swap_pane_down", "commands-swap-pane-down", mac: [], linux: [];
    ResizeLeft, "resize_pane_left", "commands-resize-pane-left", mac: [], linux: [];
    ResizeRight, "resize_pane_right", "commands-resize-pane-right", mac: [], linux: [];
    ResizeUp, "resize_pane_up", "commands-resize-pane-up", mac: [], linux: [];
    ResizeDown, "resize_pane_down", "commands-resize-pane-down", mac: [], linux: [];
    ResizeMode, "resize_mode", "commands-resize-mode", mac: [], linux: [];
    /// Evens every split in the focused tab, as a double-click on a border does.
    EqualizeSplits, "equalize_splits", "commands-equalize-splits", mac: [], linux: [];
    RenamePane, "rename_pane", "commands-rename-pane", mac: [], linux: [];
    MovePane, "move_pane", "commands-move-pane", mac: [], linux: [];
    Zoom, "toggle_zoom", "commands-toggle-zoom",
        mac: ["cmd-shift-enter"], linux: ["ctrl-shift-enter"];
    ClearPane, "clear_pane", "commands-clear-pane", mac: ["cmd-k"], linux: ["ctrl-shift-k"];
    Find, "find", "commands-find", mac: ["cmd-f"], linux: ["ctrl-shift-f"];
    FindNext, "find_next", "commands-find-next", mac: ["cmd-g"], linux: ["ctrl-shift-g"];
    FindPrevious, "find_previous", "commands-find-previous",
        mac: ["cmd-shift-g"], linux: ["ctrl-alt-shift-g"];
    CopyMode, "copy_mode", "commands-copy-mode", mac: ["cmd-shift-c"], linux: ["ctrl-alt-shift-c"];
    EditScrollback, "edit_scrollback", "commands-edit-scrollback", mac: [], linux: [];
    ClosePane, "close_pane", "commands-close-pane", mac: ["cmd-w"], linux: ["ctrl-shift-w"];
    CloseTab, "close_tab", "commands-close-tab", mac: ["cmd-shift-w"], linux: ["ctrl-alt-shift-w"];
    /// Reopens the focused workspace's most recently closed tab.
    // Ctrl-Shift-T is New Tab here, so its partner gains Alt.
    ReopenClosedTab, "reopen_closed_tab", "commands-reopen-closed-tab",
        mac: ["cmd-shift-t"], linux: ["ctrl-alt-shift-t"];
    TabNumber(1 2 3 4 5 6 7 8 9), "focus_tab_", "commands-focus-tab",
        mac: ["cmd-"], linux: ["ctrl-shift-"];
    PreviousAgent, "previous_agent", "commands-previous-agent", mac: [], linux: [];
    NextAgent, "next_agent", "commands-next-agent", mac: [], linux: [];
    AgentNumber(1 2 3 4 5 6 7 8 9), "focus_agent_", "commands-focus-agent", mac: [], linux: [];
    ToggleSidebar, "toggle_sidebar", "commands-toggle-sidebar",
        mac: ["cmd-b"], linux: ["ctrl-shift-b"];
    ToggleStatusBar, "toggle_status_bar", "commands-toggle-status-bar",
        mac: ["cmd-alt-b"], linux: ["ctrl-alt-b"];
    /// Moves keyboard focus to the next of the terminal, the title bar, the
    /// sidebar, and the status bar, where Tab moves between controls.
    // F6 alone is Windows' region key; on a Mac it stays the app's.
    // Windows' own key for moving between a window's regions.
    FocusNextRegion, "focus_next_region", "commands-focus-next-region",
        mac: ["cmd-f6"], linux: ["f6"];
    FocusPreviousRegion, "focus_previous_region", "commands-focus-previous-region",
        mac: ["cmd-shift-f6"], linux: ["shift-f6"];
    /// Hides the window's chrome and every group but the one in use.
    ToggleZenMode, "toggle_zen_mode", "commands-toggle-zen-mode",
        mac: ["ctrl-cmd-z"], linux: ["ctrl-alt-z"];
    DevicesOverview, "devices_overview", "commands-devices-overview", mac: [], linux: [];
    /// Starts a shell in herdr-core (Herdr 2) and shows it in a tab.
    NewCoreTerminal, "new_core_terminal", "commands-new-core-terminal", mac: [], linux: [];
    /// Opens a watch-only view of the focused herdr-core pane in a new tab.
    MirrorCorePane, "mirror_core_pane", "commands-mirror-core-pane", mac: [], linux: [];
    // GPUI drops Shift from punctuation: the physical Ctrl-Shift keys
    // arrive as their symbols on Linux (for example Ctrl-Shift-- as Ctrl-_).
    IncreaseFontSize, "increase_font_size", "commands-increase-font-size",
        mac: ["cmd-=", "cmd-+"], linux: ["ctrl-shift-=", "ctrl-shift-+", "ctrl-+"];
    DecreaseFontSize, "decrease_font_size", "commands-decrease-font-size",
        mac: ["cmd--"], linux: ["ctrl-shift--", "ctrl-_"];
    ResetFontSize, "reset_font_size", "commands-reset-font-size",
        mac: ["cmd-0"], linux: ["ctrl-shift-0", "ctrl-)"];
    Settings, "settings", "commands-settings", mac: ["cmd-,"], linux: ["ctrl-shift-,"];
    ReloadConfig, "reload_config", "commands-reload-config", mac: [], linux: [];
    Keybinds, "keybindings", "commands-keybindings", mac: ["cmd-/"], linux: ["ctrl-shift-/"];
    Sessions, "sessions", "commands-sessions", mac: ["cmd-shift-s"], linux: ["ctrl-shift-s"];
    Themes, "themes", "commands-themes", mac: [], linux: [];
    WorkspacePicker, "workspace_picker", "commands-workspace-picker",
        mac: ["cmd-p"], linux: ["ctrl-alt-shift-p"];
    Palette, "command_palette", "commands-command-palette",
        mac: ["cmd-shift-p"], linux: ["ctrl-shift-p"];
    /// Reopen the palette as it was last left: query, filters, and selection.
    ResumePalette, "resume_palette", "commands-resume-palette",
        mac: ["cmd-shift-r"], linux: ["ctrl-alt-shift-r"];
    Detach, "detach", "commands-detach", mac: [], linux: [];
    Reconnect, "reconnect", "commands-reconnect", mac: [], linux: [];
    Quit, "quit", "commands-quit", mac: ["cmd-q"], linux: ["ctrl-shift-q"];
    CheckForUpdates, "check-for-updates", "commands-check-for-updates", mac: [], linux: [];
    About, "about", "commands-about", mac: [], linux: [];
    InstallBrowserSkill, "install_browser_skill", "commands-install-browser-skill",
        mac: [], linux: [];
    SplitEditor, "split_editor", "commands-split-editor", mac: ["cmd-\\"], linux: ["ctrl-shift-\\"];
    ToggleFullScreen, "toggle_full_screen", "commands-toggle-full-screen",
        mac: ["ctrl-cmd-f"], linux: ["f11"];
    CycleWindows, "cycle_windows", "commands-cycle-windows",
        mac: ["cmd-`"], linux: ["ctrl-shift-`"];
    /// Closes the window it runs in, leaving the daemon and its terminals.
    // Tiling compositors such as Hyprland draw no close button.
    CloseWindow, "close_window", "commands-close-window",
        mac: ["cmd-alt-shift-w"], linux: ["ctrl-alt-w"];
}
