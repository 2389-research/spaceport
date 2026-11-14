//! `status`, `app`, `window`, `ui`, and `run`.
use super::{Context, Output, send, unexpected};
use crate::control::protocol::{
    Request, Response,
    app::{
        AppOp, CommandOp, CoreCatalog, CoreCatalogState, Reply, Status, UiOp, WindowInfo, WindowOp,
    },
};
use std::fmt::Write as _;

fn describe_window(window: &WindowInfo, text: &mut String) {
    let _ = writeln!(
        text,
        "{}{} {} ({}x{}{})",
        if window.active { "*" } else { " " },
        window.id,
        window.title,
        window.width.round(),
        window.height.round(),
        if window.fullscreen {
            ", full screen"
        } else {
            ""
        },
    );
    if let Some(workspace) = &window.workspace {
        let _ = writeln!(text, "    workspace {workspace}");
    }
    if let Some(core) = &window.core_workspace {
        let _ = write!(
            text,
            "    core workspace {} (hub {}, document {}",
            core.group_id, core.hub_core_id, core.document_name
        );
        if let Some(machine) = &core.machine_id {
            let _ = write!(text, ", machine {machine}");
        }
        if !core.current {
            let _ = write!(text, ", stale");
        }
        let _ = writeln!(text, ")");
    }
    for host in &window.hosts {
        let _ = writeln!(
            text,
            "    {} {} ({}): {}",
            if host.selected { ">" } else { " " },
            host.label,
            host.id,
            host.status
        );
    }
}

pub(super) fn describe_core_catalog(catalog: &CoreCatalog) -> String {
    let epoch = catalog
        .epoch
        .map_or_else(|| "none".to_owned(), |epoch| epoch.to_string());
    let detail = match catalog.state {
        CoreCatalogState::NotStarted => "not started".to_owned(),
        CoreCatalogState::Synchronizing => "synchronizing".to_owned(),
        CoreCatalogState::Current => catalog.machine_count.map_or_else(
            || "current; count unavailable".to_owned(),
            |count| format!("current; {count} machines"),
        ),
        CoreCatalogState::Disconnected => "disconnected; catalog stale".to_owned(),
        CoreCatalogState::Unavailable => format!(
            "unavailable ({})",
            sheprd_client::text::safe_text(catalog.error.as_deref().unwrap_or("unknown"), 160)
        ),
    };
    format!("Core machine catalog, epoch {epoch}: {detail}")
}

pub(super) fn status(context: &Context) -> crate::Result<Output> {
    let response = send(&Request::Status(context.caller()?))?;
    let Response::Status(status) = response else {
        return Err(super::rejected(response));
    };
    let Status {
        version,
        protocol,
        pid,
        windows,
        core_catalog,
    } = &status;
    let mut text =
        format!("Spaceport {version} is running (pid {pid}, control protocol {protocol})\n");
    if *protocol != crate::control::protocol::PROTOCOL_VERSION {
        let _ = writeln!(
            text,
            "This command line speaks protocol {}; update the older of the two.",
            crate::control::protocol::PROTOCOL_VERSION
        );
    }
    let _ = writeln!(text, "{}", describe_core_catalog(core_catalog));
    if windows.is_empty() {
        text.push_str("No window is open\n");
    }
    for window in windows {
        describe_window(window, &mut text);
    }
    Output::data(text.trim_end(), &status)
}

pub(super) fn app(op: AppOp, context: &Context) -> crate::Result<Output> {
    context.act(Request::App, op)
}

pub(super) fn window(op: WindowOp, context: &Context) -> crate::Result<Output> {
    // Focusing answers only `Done`.
    if op == WindowOp::Focus {
        return context.act(Request::Window, op);
    }
    match context.call(Request::Window, op)? {
        Response::Done { message } => Ok(Output::message(message)),
        Response::App {
            reply: Reply::Windows { windows },
        } => {
            let mut text = String::new();
            for window in &windows {
                describe_window(window, &mut text);
            }
            if windows.is_empty() {
                text.push_str("No window is open");
            }
            Output::data(text.trim_end(), &windows)
        }
        Response::App {
            reply: Reply::Size { width, height },
        } => Output::data(
            format!("{}x{}", width.round(), height.round()),
            &serde_json::json!({ "width": width, "height": height }),
        ),
        response => Err(unexpected(response)),
    }
}

pub(super) fn ui(op: UiOp, context: &Context) -> crate::Result<Output> {
    match context.call(Request::Ui, op)? {
        Response::Done { message } => Ok(Output::message(message)),
        Response::App {
            reply: Reply::FontSize { points },
        } => Output::data(
            format!("Terminal font size {points}"),
            &serde_json::json!({ "points": points }),
        ),
        response => Err(unexpected(response)),
    }
}

pub(super) fn run_command(name: &str, context: &Context) -> crate::Result<Output> {
    if !crate::controls::COMMANDS
        .iter()
        .any(|info| info.name == name)
    {
        return Err(crate::control::Error::Rejected {
            code: crate::control::ErrorCode::NotFound,
            message: format!("No command is named {name}; `spaceport commands` lists them"),
        }
        .into());
    }
    context.act(
        Request::Command,
        CommandOp::Run {
            name: name.to_owned(),
        },
    )
}

#[cfg(test)]
mod tests;
