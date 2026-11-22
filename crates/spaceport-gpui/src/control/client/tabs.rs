//! `tab` and `devices`.

use super::{Context, Output, unexpected};
use crate::control::protocol::{
    Request, Response,
    tabs::{Reply, TabInfo, TabOp},
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

pub(super) fn list(context: &Context) -> crate::Result<Output> {
    match context.call(Request::Tab, TabOp::List {})? {
        Response::Tab {
            reply: Reply::Tabs { tabs },
        } => {
            if tabs.is_empty() {
                return Output::data("No app tabs are open", &tabs);
            }
            Output::data(describe(&tabs), &tabs)
        }
        response => Err(unexpected(response)),
    }
}

pub(super) fn tab(op: TabOp, context: &Context) -> crate::Result<Output> {
    match op {
        TabOp::List {} => list(context),
        TabOp::Devices => devices(context),
        op @ (TabOp::Focus { .. } | TabOp::Close { .. }) => context.act(Request::Tab, op),
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

#[cfg(test)]
mod tests;
