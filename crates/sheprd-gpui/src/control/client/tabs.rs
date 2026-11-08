//! `tab` and `devices`.

use super::{Context, Output, unexpected};
use crate::{
    cli::control::TabCommand,
    control::protocol::{
        Request, Response,
        tabs::{Reply, TabInfo, TabOp},
    },
};
use std::fmt::Write as _;

/// One line per tab: its ID, kind, title, what it shows, and where.
fn describe(tabs: &[TabInfo]) -> String {
    let mut text = String::new();
    for tab in tabs {
        let _ = write!(text, "{}\t{}\t{}", tab.id, tab.kind.name(), tab.title);
        if let Some(location) = &tab.location {
            let _ = write!(text, "\t{location}");
        }
        let _ = write!(text, "\t(workspace {}", tab.workspace);
        if let Some(pane) = &tab.opened_by {
            let _ = write!(text, ", opened by {pane}");
        }
        let _ = writeln!(text, ")");
    }
    text.trim_end().to_owned()
}

pub(super) fn list(pages: bool, context: &Context) -> crate::Result<Output> {
    match context.call(Request::Tab, TabOp::List { pages })? {
        Response::Tab {
            reply: Reply::Tabs { tabs },
        } => {
            if tabs.is_empty() {
                let text = if pages {
                    "No page tabs are open"
                } else {
                    "No app tabs are open"
                };
                return Output::data(text, &tabs);
            }
            Output::data(describe(&tabs), &tabs)
        }
        response => Err(unexpected(response)),
    }
}

pub(super) fn tab(command: TabCommand, context: &Context) -> crate::Result<Output> {
    match command {
        TabCommand::List => list(false, context),
        TabCommand::Focus { id } => context.act(Request::Tab, TabOp::Focus { id }),
        TabCommand::Close { id } => context.act(Request::Tab, TabOp::Close { id }),
    }
}

/// What `Opened` says, for a person.
fn opened(what: &str, response: Response) -> crate::Result<Output> {
    match response {
        Response::Tab { reply } => match &reply {
            Reply::Opened { id, workspace } => Output::data(
                format!("Opened {what} in tab {id} of workspace {workspace}"),
                &reply,
            ),
            _ => Err(unexpected(Response::Tab { reply })),
        },
        Response::Done { message } => Ok(Output::message(message)),
        response => Err(unexpected(response)),
    }
}

pub(super) fn devices(context: &Context) -> crate::Result<Output> {
    let response = context.call(Request::Tab, TabOp::Devices)?;
    opened("the Devices overview", response)
}

pub(super) fn px0(context: &Context) -> crate::Result<Output> {
    opened("px0", context.call(Request::Tab, TabOp::Px0)?)
}

#[cfg(test)]
mod tests;
