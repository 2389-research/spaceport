//! The work flows that end in a dialog of the app's own: running a worktree
//! script and fanning out. Each opens the flow as the caller's workspace
//! menu would, already pointed at
//! the caller's checkout, and leaves the consequential step to the user:
//! launching and trusting a script wait for them.
use super::{find, repository, selected_workspace};
use crate::{
    SheprdWindow,
    control::protocol::{
        Caller, ErrorCode, Response,
        work::{FanOutRequest, Script},
    },
    dialog_input::DialogInput,
    menu::Page,
    teleport::AgentKind,
    worktree_scripts::{Launch, ScriptKind},
};
use gpui::{App, Context, Point, Window};

impl From<Script> for ScriptKind {
    fn from(script: Script) -> Self {
        match script {
            Script::Setup => Self::Setup,
            Script::Run => Self::Run,
            Script::Archive => Self::Archive,
        }
    }
}

fn invalid(message: impl Into<String>) -> Response {
    Response::error(ErrorCode::InvalidRequest, message)
}

/// Brings the window forward, where the user confirms. Deferred: activating
/// a window updates it, which cannot happen inside this update.
pub(in crate::control::server) fn bring_forward(window: &Window, cx: &mut App) {
    let handle = window.window_handle();
    cx.activate(true);
    cx.defer(move |cx| {
        let _ = handle.update(cx, |_, window, _| window.activate_window());
    });
}

/// Opens the caller's workspace menu, the way every flow here starts.
fn open_menu(
    view: &mut SheprdWindow,
    caller: &Caller,
    window: &mut Window,
    cx: &mut Context<SheprdWindow>,
) -> Result<String, Response> {
    let workspace = selected_workspace(view, caller)?;
    view.open_workspace_menu(&workspace, Point::default(), window, cx);
    if view.menu.page != Some(Page::Workspace) {
        return Err(Response::error(
            ErrorCode::Failed,
            "The workspace's menu did not open",
        ));
    }
    Ok(workspace)
}

pub(super) fn script(
    view: &mut SheprdWindow,
    caller: &Caller,
    kind: Script,
    cx: &mut Context<SheprdWindow>,
) -> Response {
    let kind = ScriptKind::from(kind);
    if kind == ScriptKind::Archive {
        return Response::error(
            ErrorCode::Unsupported,
            "The archive script runs when a worktree is removed; remove it from its workspace menu in Spaceport",
        );
    }
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
        force: false,
        requested: true,
    })
}

/// An agent kind by the name Herdr reports, ignoring case.
pub(super) fn agent_kind(name: &str) -> Option<AgentKind> {
    let name = name.to_ascii_lowercase();
    AgentKind::parse(&name).or((name == "antigravity").then_some(AgentKind::Antigravity))
}

/// The agents asked for, merged by kind in the order first named.
pub(super) fn agents(asked: &[(String, u8)]) -> Result<Vec<(AgentKind, usize)>, Response> {
    let mut agents: Vec<(AgentKind, usize)> = Vec::new();
    for (name, count) in asked {
        let Some(kind) = agent_kind(name) else {
            let known: Vec<_> = AgentKind::ALL.iter().map(|kind| kind.name()).collect();
            return Err(invalid(format!(
                "No agent is named {name}; known agents: {}",
                known.join(", ")
            )));
        };
        match agents.iter_mut().find(|(picked, _)| *picked == kind) {
            Some((_, total)) => *total += usize::from(*count),
            None => agents.push((kind, usize::from(*count))),
        }
    }
    let total: usize = agents.iter().map(|(_, count)| count).sum();
    if total > crate::fan_out::MAX_LANES {
        return Err(invalid(format!(
            "A fan-out runs at most {} agents",
            crate::fan_out::MAX_LANES
        )));
    }
    Ok(agents)
}

/// The prompt as the dialog's one-line field holds it.
pub(super) fn prompt_text(prompt: &str) -> String {
    prompt
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .filter(|c| !crate::notifications::unsafe_char(*c))
        .collect::<String>()
        .trim()
        .to_owned()
}

pub(super) fn fan_out(
    view: &mut SheprdWindow,
    caller: &Caller,
    request: FanOutRequest,
    window: &mut Window,
    cx: &mut Context<SheprdWindow>,
) -> Response {
    let agents = match agents(&request.agents) {
        Ok(agents) => agents,
        Err(response) => return response,
    };
    let prompt = prompt_text(&request.prompt);
    if prompt.is_empty() {
        return invalid("Write a prompt");
    }
    if view
        .fan_out
        .as_ref()
        .is_some_and(|fan_out| !fan_out.composing())
    {
        return Response::error(
            ErrorCode::Busy,
            "This window's last fan-out is still open for comparison; keep a lane or close it from the workspace menu first",
        );
    }
    let workspace = match open_menu(view, caller, window, cx) {
        Ok(workspace) => workspace,
        Err(response) => return response,
    };
    if view.fan_out_item().is_none() {
        view.dismiss_menu(window, cx);
        return invalid(
            "Fan-out needs a Git workspace on this machine or an SSH host, from Linux or macOS",
        );
    }
    let repo = find(view, caller)
        .map(|found| repository(found.workspace).to_owned())
        .unwrap_or(workspace);
    view.open_fan_out(window, cx);
    let opened = view.menu.page == Some(Page::FanOut)
        && view
            .fan_out
            .as_ref()
            .is_some_and(|fan_out| fan_out.composing());
    if !opened {
        // As every other refusal here, leave no menu behind.
        view.dismiss_menu(window, cx);
        return Response::error(ErrorCode::Failed, "The fan-out dialog did not open");
    }
    let Some(fan_out) = view.fan_out.as_mut() else {
        return Response::error(ErrorCode::Failed, "The fan-out dialog did not open");
    };
    let picked: usize = agents.iter().map(|(_, count)| count).sum();
    fan_out.want(agents);
    let mut input = DialogInput::new(prompt);
    let end = input.text.len();
    input.selection = end..end;
    view.menu.input = Some(input);
    cx.notify();
    bring_forward(window, cx);
    Response::done(format!(
        "Opened fan-out for {repo} with {picked} agent(s) chosen; start it in Spaceport"
    ))
}
