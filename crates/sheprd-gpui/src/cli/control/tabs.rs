//! `tab` and `devices`.
use super::{Args, CliError, ControlCommand, TabOp, unknown};

fn id(args: &mut Args) -> Result<u64, CliError> {
    args.required_text("ID")?
        .parse()
        .map_err(|_| CliError::InvalidValue("ID"))
}

pub(super) const TAB_USAGE: &str = "spaceport tab [list]\n    The app's own tabs: pages, Devices, Core terminals.\n    Terminal tabs belong to Herdr: use `herdr tab`.\nspaceport tab focus|close ID\n    Show a tab, switching to its workspace, or close it.";

pub(super) fn tab(args: &mut Args) -> Result<TabOp, CliError> {
    let Some(verb) = args.verb() else {
        return Ok(TabOp::List {});
    };
    Ok(match verb.to_str() {
        Some("list") => TabOp::List {},
        Some("focus") => TabOp::Focus { id: id(args)? },
        Some("close") => TabOp::Close { id: id(args)? },
        _ => return Err(unknown("tab", verb)),
    })
}

pub(super) const DEVICES_USAGE: &str =
    "spaceport devices [open]\n    Open the Devices overview tab in the caller's workspace.";

pub(super) fn devices(args: &mut Args) -> Result<ControlCommand, CliError> {
    open_only(args)?;
    Ok(ControlCommand::Devices)
}

/// A family whose only verb is `open`, which may be left out.
fn open_only(args: &mut Args) -> Result<(), CliError> {
    match args.verb() {
        None => Ok(()),
        Some(verb) if verb == "open" => Ok(()),
        Some(verb) => Err(unknown(args.name(), verb)),
    }
}
