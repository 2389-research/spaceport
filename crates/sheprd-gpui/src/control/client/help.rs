// ABOUTME: Builds command usage from the shared control family catalog.
// ABOUTME: Documents output options, exact targeting and exit status categories.
//! Usage text for every control command.
use crate::cli::control::FAMILIES;

const COMMON: &str = "\
Commands accept --json for JSON output; events --follow emits JSON lines.
App commands accept --window ID,
acting in that window (IDs come from `window list`). By default a command
acts in the window showing the caller's workspace (HERDR_WORKSPACE_ID), and
exits 5 when no window shows it; outside Herdr, in the frontmost window.
Local commands reject --window. Core pane window controls require an explicit target.
Exit status: 0 done, 1 refused, 2 invalid arguments, 3 app unavailable,
4 nothing yet (no notes, no events, no chat panes, no ports, no usage, no log,
a key not set, or the wait not met), 5 not found, 6 busy (a dialog is open),
7 timed out, 8 unsupported on this platform or by this app version.";

/// Usage for one family, or the list of families.
pub(crate) fn usage(name: Option<&str>) -> String {
    match name {
        Some(name) => {
            let usage = FAMILIES
                .iter()
                .find(|family| family.name == name)
                .map_or("", |family| family.usage);
            format!("{usage}\n\n{COMMON}")
        }
        None => {
            let mut text = String::from("spaceport COMMAND ... [--json] [--window ID]\n\n");
            for family in FAMILIES {
                text.push_str(&format!("  {:<14}{}\n", family.name, family.summary));
            }
            text.push_str("\nRun `spaceport COMMAND --help` for one command's usage.\n\n");
            text.push_str(COMMON);
            text
        }
    }
}
