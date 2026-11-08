//! Shared host facts for the remaining orchestration and pane carry routes.
//! Candidate ranking and Teleport targets still serve those excluded callers.

use super::{Candidate, History, Load, Repository};
use crate::{
    SheprdWindow,
    teleport::{FreshOrigin, HostRepositories, Place, host_for, open_repositories},
};
use gpui::App;
use sheprd_client::protocol::{AgentStatus, ClientShellSnapshot};

/// Host scripts need a POSIX client; see `sheprd_client::run_script`.
const SCRIPTABLE_CLIENT: bool = cfg!(any(target_os = "linux", target_os = "macos"));

fn working(snapshot: &ClientShellSnapshot) -> u32 {
    snapshot
        .agents
        .iter()
        .filter(|agent| agent.agent_status == AgentStatus::Working)
        .count() as u32
}

fn repository(snapshot: Option<&ClientShellSnapshot>, repo: &str) -> Repository {
    let Some(snapshot) = snapshot else {
        return Repository::Unknown;
    };
    let present = snapshot.workspaces.iter().any(|workspace| {
        workspace
            .worktree
            .as_ref()
            .is_some_and(|tree| tree.label == repo)
    });
    if present {
        Repository::Present
    } else {
        Repository::Missing
    }
}

impl SheprdWindow {
    /// The connected snapshot of endpoint `index`, reading the selected one
    /// from the window's own copy.
    pub(crate) fn endpoint_snapshot(&self, index: usize) -> Option<&ClientShellSnapshot> {
        let live = if index == self.selected_endpoint {
            &self.live
        } else {
            &self.endpoints[index].live
        };
        live.snapshot
            .as_deref()
            .filter(|_| live.status.is_connected())
    }

    /// Every host a new checkout of `repo` could go to: the one shown, and
    /// every other enabled host this client can script. Empty when the
    /// shown host cannot be scripted, since its commit could not be shipped.
    pub(crate) fn dispatch_candidates(&self, repo: &str, cx: &App) -> Vec<Candidate> {
        let current = &self.endpoints[self.selected_endpoint];
        if !SCRIPTABLE_CLIENT || host_for(&current.connection.target).is_err() {
            return Vec::new();
        }
        let history = cx.try_global::<History>();
        self.endpoints
            .iter()
            .enumerate()
            .filter(|(index, endpoint)| {
                *index == self.selected_endpoint
                    || (endpoint.enabled && host_for(&endpoint.connection.target).is_ok())
            })
            .map(|(index, endpoint)| {
                let snapshot = self.endpoint_snapshot(index);
                let load = crate::usage::Host::load_of(&endpoint.connection.target)
                    .and_then(|host| self.system_load.get(&host))
                    .and_then(|reading| reading.latest())
                    .map(|sample| Load {
                        cpu: sample.cpu,
                        cores: sample.cores,
                        load5: sample.load.map(|load| load[1]),
                        memory: sample.memory.map(|memory| memory.percent()),
                    });
                Candidate {
                    endpoint_id: endpoint.id.clone(),
                    label: endpoint.label.clone(),
                    online: snapshot.is_some(),
                    current: index == self.selected_endpoint,
                    load,
                    working: snapshot.map_or(0, working),
                    repository: repository(snapshot, repo),
                    picks: history.map_or(0, |history| history.picks(repo, &endpoint.id)),
                }
            })
            .collect()
    }

    /// The repository a dispatch branches from, on the shown host.
    pub(crate) fn dispatch_origin(
        &self,
        workspace_id: &str,
        repo_key: &str,
        repo_label: &str,
    ) -> Option<FreshOrigin> {
        let selected = &self.endpoints[self.selected_endpoint];
        Some(FreshOrigin {
            place: Place {
                endpoint_id: selected.id.clone(),
                label: selected.label.clone(),
                host: host_for(&selected.connection.target).ok()?,
            },
            workspace_id: workspace_id.to_owned(),
            repo_key: repo_key.to_owned(),
            repo_label: repo_label.to_owned(),
        })
    }

    /// The host `endpoint_id` as Teleport's scripts address it, with the
    /// repositories its snapshot shows when it is connected.
    pub(crate) fn dispatch_destination(&self, endpoint_id: &str) -> Option<HostRepositories> {
        let index = self.endpoints.iter().position(|e| e.id == endpoint_id)?;
        let endpoint = &self.endpoints[index];
        Some(HostRepositories {
            place: Place {
                endpoint_id: endpoint.id.clone(),
                label: endpoint.label.clone(),
                host: host_for(&endpoint.connection.target).ok()?,
            },
            repositories: self.endpoint_snapshot(index).map(open_repositories),
            retired: Vec::new(),
        })
    }
}
