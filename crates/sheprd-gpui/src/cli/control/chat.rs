//! `chat`: the native views of `herdr-acp` chats over their panes.
use super::{Args, CliError, Switch, unknown};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChatCommand {
    /// The window's chat panes and how each shows.
    Status,
    /// Open a tab running a chat with `agent`.
    New {
        agent: sheprd_chat_protocol::Agent,
        /// The pane whose workspace and folder it starts in, else the caller's.
        pane: Option<String>,
    },
    /// Show the chat view over a pane (`On`), or its terminal (`Off`).
    View {
        switch: Switch,
        /// The pane, else the caller's own.
        pane: Option<String>,
    },
}

pub(super) const CHAT_USAGE: &str = "spaceport chat [status]\n    The window's sheprd-acp chat panes on this machine or an SSH host: pane, agent, status,\n    and whether the native chat view or the terminal shows. Exits 4 when\n    there are none.\nspaceport chat show|hide|toggle [--pane ID]\n    Show the native chat view over the pane, or its terminal, for this\n    window. The pane is the caller's own, else the focused one. Exits 5 when\n    it runs no chat the window can show.\nspaceport chat new AGENT [--pane ID]\n    Open a tab running a sheprd-acp chat with AGENT (claude, codex, copilot,\n    gemini, opencode, or pi) in the workspace and folder of the pane, else the\n    caller's own. On an SSH host the tab runs that host's sheprd-acp, from\n    ~/.herdr/bin or PATH. With no Herdr pane to go by, a Herdr 2 tab in the\n    active group splits its pane in use with the chat instead. The chat view\n    attaches once the program starts.\n    Exits 8 when ACP chats are off in Settings, sheprd-acp is missing here,\n    the host cannot be reached, or the agent is not installed there.";

pub(super) fn chat(args: &mut Args) -> Result<ChatCommand, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(ChatCommand::Status);
    };
    let switch = match verb.to_str() {
        Some("status" | "list") => return Ok(ChatCommand::Status),
        Some("new") => {
            let pane = args.text_option("--pane")?;
            let agent = args
                .required_text("an agent, such as claude")?
                .parse()
                .map_err(|_| CliError::InvalidValue("AGENT"))?;
            return Ok(ChatCommand::New { agent, pane });
        }
        Some("show") => Switch::On,
        Some("hide") => Switch::Off,
        Some("toggle") => Switch::Toggle,
        _ => return Err(unknown("chat", verb)),
    };
    Ok(ChatCommand::View {
        switch,
        pane: args.text_option("--pane")?,
    })
}
