//! `app`, `window`, and `ui`.
use super::{AppOp, Args, CliError, FontSize, Size, UiOp, WindowOp, switch, unknown};

pub(super) const APP_USAGE: &str = "spaceport app focus|hide|quit\n    Bring the app forward, hide it, or quit it. Quitting leaves Herdr\n    and its terminals running.";

pub(super) fn app(args: &mut Args) -> Result<AppOp, CliError> {
    let verb = args.verb().ok_or(CliError::MissingArgument {
        command: "app",
        what: "focus|hide|quit",
    })?;
    Ok(match verb.to_str() {
        Some("focus") => AppOp::Focus,
        Some("hide") => AppOp::Hide,
        Some("quit") => AppOp::Quit,
        _ => return Err(unknown("app", verb)),
    })
}

pub(super) const WINDOW_USAGE: &str = "spaceport window [list]\n    The app's windows, with IDs for --window.\nspaceport window new\n    Open another window on the selected window's exact Core group or legacy\n    launch target, the caller's host, or the frontmost/default target. A\n    stale Core group is refused before the window opens.\nspaceport window focus ID\nspaceport window fullscreen [on|off|toggle]\nspaceport window size [WIDTHxHEIGHT]\n    Print the window's size, or resize it. Windows cannot be moved.\nspaceport window detach|reconnect\n    Detach from, or reconnect to, the window's selected host.";

pub(super) fn window(args: &mut Args) -> Result<WindowOp, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(WindowOp::List);
    };
    Ok(match verb.to_str() {
        Some("list") => WindowOp::List,
        Some("new") => WindowOp::New {},
        Some("focus") => {
            // The operation carries no window: the ID is the window the
            // command acts in, whatever `--window` named.
            let id = args
                .required_text("ID")?
                .parse()
                .map_err(|_| CliError::InvalidValue("ID"))?;
            args.set_window(Some(id));
            WindowOp::Focus
        }
        Some("fullscreen") => WindowOp::Fullscreen {
            switch: switch(args.positional())?,
        },
        Some("size") => WindowOp::Size {
            size: args.positional().map(size).transpose()?,
        },
        Some("detach") => WindowOp::Detach,
        Some("reconnect") => WindowOp::Reconnect,
        _ => return Err(unknown("window", verb)),
    })
}

/// `WIDTHxHEIGHT` in points, each within what a screen can hold.
fn size(text: std::ffi::OsString) -> Result<Size, CliError> {
    text.to_str()
        .and_then(|text| text.split_once(['x', 'X']))
        .and_then(|(width, height)| Some((width.parse().ok()?, height.parse().ok()?)))
        .filter(|(width, height)| (200..=16_384).contains(width) && (150..=16_384).contains(height))
        .map(|(width, height)| Size { width, height })
        .ok_or(CliError::InvalidValue("WIDTHxHEIGHT"))
}

/// The panel names `ui panel` takes, as its errors and help name them.
const PANELS: &str = "workspaces|overview|automations|usage";

pub(super) const UI_USAGE: &str = "spaceport ui sidebar [on|off|toggle]\nspaceport ui statusbar [on|off|toggle]\nspaceport ui font-size +|-|reset|POINTS\nspaceport ui panel workspaces|overview|automations|usage\n    For this window and session; settings are changed with `config`.";

pub(super) fn ui(args: &mut Args) -> Result<UiOp, CliError> {
    let verb = args.verb().ok_or(CliError::MissingArgument {
        command: "ui",
        what: "sidebar|statusbar|font-size|panel",
    })?;
    Ok(match verb.to_str() {
        Some("sidebar") => UiOp::Sidebar {
            switch: switch(args.positional())?,
        },
        Some("statusbar" | "status-bar") => UiOp::StatusBar {
            switch: switch(args.positional())?,
        },
        Some("panel") => {
            let name = args.required_text(PANELS)?;
            UiOp::Panel {
                panel: crate::sidebar::ActivityPanel::parse(&name)
                    .ok_or(CliError::InvalidValue(PANELS))?,
            }
        }
        Some("font-size") => {
            let value = args.required_text("+|-|reset|POINTS")?;
            let size = match value.as_str() {
                "+" | "larger" => FontSize::Larger,
                "-" | "smaller" => FontSize::Smaller,
                "reset" => FontSize::Reset,
                points => FontSize::Points {
                    points: points
                        .parse()
                        .ok()
                        .filter(|points| (8..=48).contains(points))
                        .ok_or(CliError::InvalidValue("POINTS (8-48)"))?,
                },
            };
            UiOp::FontSize { size }
        }
        _ => return Err(unknown("ui", verb)),
    })
}
