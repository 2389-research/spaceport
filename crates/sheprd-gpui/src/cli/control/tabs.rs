//! `tab` and `devices`.
use super::{Args, CliError, unknown};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabCommand {
    List,
    Focus { id: u64 },
    Close { id: u64 },
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
