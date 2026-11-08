//! `tab`, `devices`, and `orchestrator`.
use super::{Args, CliError, unknown};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabCommand {
    List,
    Focus { id: u64 },
    Close { id: u64 },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Items {
    #[default]
    All,
    Issues,
    PullRequests,
    Tasks,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrchestratorCommand {
    Open {
        repo: Option<PathBuf>,
    },
    /// The issues and pull requests the orchestrator last synced, read from
    /// its local database.
    Items {
        repo: Option<PathBuf>,
        items: Items,
    },
}

fn id(args: &mut Args) -> Result<u64, CliError> {
    args.required_text("ID")?
        .parse()
        .map_err(|_| CliError::InvalidValue("ID"))
}

pub(super) fn tab(args: &mut Args) -> Result<TabCommand, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(TabCommand::List);
    };
    Ok(match verb.to_str() {
        Some("list") => TabCommand::List,
        Some("focus") => TabCommand::Focus { id: id(args)? },
        Some("close") => TabCommand::Close { id: id(args)? },
        _ => return Err(unknown("tab", verb)),
    })
}

/// A family whose only verb is `open`, which may be left out.
pub(super) fn open_only(args: &mut Args) -> Result<(), CliError> {
    match args.verb() {
        None => Ok(()),
        Some(verb) if verb == "open" => Ok(()),
        Some(verb) => Err(unknown(args.name(), verb)),
    }
}

pub(super) fn orchestrator(args: &mut Args) -> Result<OrchestratorCommand, CliError> {
    // The option first, so `orchestrator --repo PATH` opens PATH.
    let repo = args.option("--repo")?.map(PathBuf::from);
    let verb = args.verb();
    Ok(match verb.as_ref().and_then(|verb| verb.to_str()) {
        None | Some("open") => OrchestratorCommand::Open { repo },
        Some("items") => OrchestratorCommand::Items {
            repo,
            items: Items::All,
        },
        Some("issues") => OrchestratorCommand::Items {
            repo,
            items: Items::Issues,
        },
        Some("prs") => OrchestratorCommand::Items {
            repo,
            items: Items::PullRequests,
        },
        Some("tasks") => OrchestratorCommand::Items {
            repo,
            items: Items::Tasks,
        },
        _ => return Err(unknown("orchestrator", verb.unwrap_or_default())),
    })
}
