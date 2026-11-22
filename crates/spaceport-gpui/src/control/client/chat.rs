//! `chat`: list the window's chat panes, or show a chat view or its terminal.
use super::{Context, Output, unexpected};
use crate::control::protocol::{
    Request, Response,
    chat::{ChatInfo, ChatOp, Reply},
};
use std::fmt::Write as _;

/// One chat pane as a line of text.
pub(super) fn describe(chat: &ChatInfo) -> String {
    let mut line = format!(
        "{}  {}  {}{}  {}",
        chat.pane_id,
        if chat.agent.is_empty() {
            "agent"
        } else {
            &chat.agent
        },
        chat.status,
        if chat.attached { "" } else { " (reconnecting)" },
        if chat.shown { "chat view" } else { "terminal" },
    );
    if let Some(model) = &chat.model {
        let _ = write!(line, "  {model}");
    }
    if let Some(context) = chat.context {
        let _ = write!(line, "  {context}% context");
    }
    if chat.permissions > 0 {
        let _ = write!(line, "  {} waiting for permission", chat.permissions);
    }
    if chat.queued > 0 {
        let _ = write!(line, "  {} queued", chat.queued);
    }
    line
}

pub(super) fn chat(op: ChatOp, context: &Context) -> crate::Result<Output> {
    match context.call(Request::Chat, op)? {
        Response::Chat {
            reply: Reply::Chats { chats },
        } => {
            let json = serde_json::json!({ "chats": chats });
            if chats.is_empty() {
                return Ok(Output::data("No chat panes", &json)?.nothing());
            }
            let text: Vec<String> = chats.iter().map(describe).collect();
            Output::data(text.join("\n"), &json)
        }
        Response::Chat {
            reply: Reply::View { pane_id, shown },
        } => Output::data(
            if shown {
                format!("{pane_id} shows its chat view")
            } else {
                format!("{pane_id} shows its terminal")
            },
            &serde_json::json!({ "pane_id": pane_id, "shown": shown }),
        ),
        Response::Chat {
            reply: Reply::Started { agent },
        } => Output::data(
            format!("Starting a {agent} chat"),
            &serde_json::json!({ "agent": agent, "started": true }),
        ),
        response => Err(unexpected(response)),
    }
}
