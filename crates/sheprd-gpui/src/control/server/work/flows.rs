//! Worktree script requests, targeted at the caller's checkout.
//! An untrusted script still waits for the user's approval in the app.
use super::{find, selected_workspace};
use crate::{
    SheprdWindow,
    control::protocol::{Caller, ErrorCode, Response, work::Script},
    worktree_scripts::{Launch, ScriptKind},
};
use gpui::Context;

impl From<Script> for ScriptKind {
    fn from(script: Script) -> Self {
        match script {
            Script::Setup => Self::Setup,
            Script::Run => Self::Run,
        }
    }
}

fn invalid(message: impl Into<String>) -> Response {
    Response::error(ErrorCode::InvalidRequest, message)
}

pub(super) fn script(
    view: &mut SheprdWindow,
    caller: &Caller,
    kind: Script,
    cx: &mut Context<SheprdWindow>,
) -> Response {
    let kind = ScriptKind::from(kind);
    let launch = match launch(view, caller, kind) {
        Ok(launch) => launch,
        Err(response) => return response,
    };
    let repo = launch.repo.clone();
    match view.start_worktree_script(launch, cx) {
        Ok(()) => Response::done(format!(
            "Starting the {} script of {repo} in a new tab; Spaceport asks first about a script it has not seen",
            kind.name()
        )),
        Err(crate::Error::WorktreeScripts(crate::worktree_scripts::Error::Busy)) => {
            Response::error(ErrorCode::Busy, "Another worktree script is still starting")
        }
        Err(error) => Response::error(ErrorCode::Failed, crate::control::describe_error(&error)),
    }
}

/// The script job for the caller's workspace, built as its menu builds one.
fn launch(view: &SheprdWindow, caller: &Caller, kind: ScriptKind) -> Result<Launch, Response> {
    selected_workspace(view, caller)?;
    let found = find(view, caller)?;
    let Some(tree) = &found.workspace.worktree else {
        return Err(invalid(crate::worktree_scripts::Error::NotGit.to_string()));
    };
    // The menu offers the setup script on linked worktrees alone.
    if kind == ScriptKind::Setup && !tree.is_linked_worktree {
        return Err(invalid(
            "The setup script runs in linked worktrees, not the main checkout",
        ));
    }
    let endpoint = &view.endpoints[view.selected_endpoint];
    Ok(Launch {
        kind,
        endpoint: (view.selection_epoch, endpoint.generation),
        endpoint_id: endpoint.id.clone(),
        boot: found.snapshot.boot_id.clone(),
        workspace: found.workspace.workspace_id.clone(),
        repo: tree.label.clone(),
        repo_key: tree.key.clone(),
        checkout: None,
        requested: true,
    })
}
