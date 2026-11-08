//! `browser ...` and `feedback`.
use super::{Context, Output, caller, notes_daemon, rejected, send, tabs, unexpected};
use crate::{
    browser::WebUrl,
    cli::control::{BrowserCommand, ZoomChange},
    control::protocol::{
        ErrorCode, FeedbackRequest, MAX_WAIT_SECONDS, Request, Response,
        tabs::{Reply, TabOp},
    },
};
use std::path::Path;

/// The explainer kit's example, which the agent skill points to.
pub(super) const EXAMPLE: &str =
    include_str!("../../../../../skills/herdr-gpui-browser/examples/instagram-fetch.html");

fn reload() -> crate::Result<Output> {
    match send(&Request::Reload(caller(None)?))? {
        Response::Reloaded { tabs: 0 } => Err(crate::control::Error::Rejected {
            code: ErrorCode::InvalidRequest,
            message: "This pane has no browser tabs open".into(),
        }
        .into()),
        Response::Reloaded { tabs } => Output::data(
            format!("Reloaded {tabs} browser tab(s)"),
            &serde_json::json!({ "tabs": tabs }),
        ),
        response => Err(rejected(response)),
    }
}

/// The user's page and review notes for this pane are kept for it whenever
/// they could not be typed into the pane.
pub(super) fn feedback(wait: u64, _: &Context) -> crate::Result<Output> {
    let pane_id = super::env_id("HERDR_PANE_ID").ok_or(crate::control::Error::Rejected {
        code: ErrorCode::InvalidRequest,
        message: "feedback runs in the Herdr pane that should receive the notes".into(),
    })?;
    let request = Request::Feedback(FeedbackRequest {
        pane_id,
        daemon_socket: Some(notes_daemon()?),
        wait_seconds: wait.min(MAX_WAIT_SECONDS),
    });
    match send(&request)? {
        // Notes keep their line breaks: they are Markdown for the agent.
        Response::Feedback { text: Some(text) } => {
            Output::data(text.trim_end(), &serde_json::json!({ "notes": text }))
        }
        Response::Feedback { text: None } => Ok(Output::data(
            "No notes from the user yet",
            &serde_json::json!({ "notes": null }),
        )?
        .nothing()),
        response => Err(rejected(response)),
    }
}

/// The text of a page tab.
fn text(id: u64, context: &Context) -> crate::Result<Output> {
    match context.call(Request::Tab, TabOp::Text { id })? {
        Response::Tab {
            reply: Reply::Text { text },
        } => Output::data(text.clone(), &serde_json::json!({ "id": id, "text": text })),
        response => Err(unexpected(response)),
    }
}

/// Zooms a page tab.
fn zoom(id: u64, change: ZoomChange, context: &Context) -> crate::Result<Output> {
    zoomed(context.call(Request::Tab, TabOp::Zoom { id, change })?)
}

/// What a zoomed tab says: the zoom it landed on.
fn zoomed(response: Response) -> crate::Result<Output> {
    match response {
        Response::Tab {
            reply: Reply::Zoom { id, percent },
        } => Output::data(
            format!("Tab {id} is zoomed to {percent}%"),
            &serde_json::json!({ "id": id, "zoom": percent }),
        ),
        response => Err(unexpected(response)),
    }
}

/// The screenshot is written by the app, so the path is made absolute here,
/// where the caller's working directory is known.
fn screenshot(id: u64, path: &Path, context: &Context) -> crate::Result<Output> {
    let path = std::path::absolute(path)?;
    let text = path
        .to_str()
        .ok_or(crate::Error::InvalidLocalPage)?
        .to_owned();
    context.act(Request::Tab, TabOp::Screenshot { id, path: text })
}

pub(super) fn run(command: BrowserCommand, context: &Context) -> crate::Result<Output> {
    match command {
        // Answered before any request; see `client::run`.
        BrowserCommand::Help | BrowserCommand::Skill | BrowserCommand::Example => {
            Ok(Output::message(""))
        }
        BrowserCommand::Reload => reload(),
        BrowserCommand::Feedback { wait } => feedback(wait, context),
        BrowserCommand::List => tabs::list(true, context),
        BrowserCommand::Close { id } => context.act(Request::Tab, TabOp::Close { id }),
        BrowserCommand::Navigate { id, url } => {
            let url = WebUrl::from_typed(&url)?;
            context.act(
                Request::Tab,
                TabOp::Navigate {
                    id,
                    url: url.as_str().to_owned(),
                },
            )
        }
        BrowserCommand::Back { id } => context.act(Request::Tab, TabOp::Back { id }),
        BrowserCommand::Forward { id } => context.act(Request::Tab, TabOp::Forward { id }),
        BrowserCommand::Text { id } => text(id, context),
        BrowserCommand::Screenshot { id, path } => screenshot(id, &path, context),
        BrowserCommand::Zoom { id, change } => zoom(id, change, context),
    }
}

#[cfg(test)]
mod tests;
