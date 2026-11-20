// ABOUTME: Owns pane-history edits for exact local and SSH runtime targets.
// ABOUTME: Preserves guarded local snapshots and remote cancellation fences.
//! Runtime pane history is separate from Spaceport's private view settings.
// i18n: migrated

use super::SettingsWindow;
use crate::herdr_settings::{Edit, RemotePaneHistory, Settings};
use crate::i18n::Localize;
use gpui::{AsyncApp, Context, WeakEntity};
use sheprd_client::{ConnectTarget, ScriptHost, SshLane};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub(super) enum HostState {
    Loading,
    Ready(History),
    /// Holds the switch at its last read value until the write reconciles.
    Saving(History),
    /// Display text with the causes, such as the SSH failure, prepared once.
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Target {
    Local,
    Ssh(String),
}

#[derive(Clone)]
pub(super) enum History {
    Local(Box<Settings>),
    Ssh {
        target: String,
        history: RemotePaneHistory,
    },
}

impl History {
    pub(super) fn enabled(&self) -> bool {
        match self {
            Self::Local(settings) => settings.pane_history,
            Self::Ssh { history, .. } => history.enabled,
        }
    }

    fn load(target: &Target, cancelled: &AtomicBool) -> crate::Result<Self> {
        let result = match target {
            Target::Local => Settings::load_runtime_history()
                .map(Box::new)
                .map(Self::Local),
            Target::Ssh(target) => {
                RemotePaneHistory::load(ScriptHost::Ssh(target, SshLane::Quick), cancelled).map(
                    |history| Self::Ssh {
                        target: target.clone(),
                        history,
                    },
                )
            }
        };
        result.map_err(crate::Error::from)
    }

    fn save(&self, enabled: bool, cancelled: &AtomicBool) -> crate::Result<Self> {
        if cancelled.load(Ordering::Acquire) {
            return Err(std::io::Error::from(std::io::ErrorKind::Interrupted).into());
        }
        let result = match self {
            Self::Local(settings) => settings
                .save(Edit::PaneHistory(enabled))
                .map(Box::new)
                .map(Self::Local),
            Self::Ssh { target, history } => history
                .save(ScriptHost::Ssh(target, SshLane::Quick), enabled, cancelled)
                .map(|history| Self::Ssh {
                    target: target.clone(),
                    history,
                }),
        };
        result.map_err(crate::Error::from)
    }
}

pub(super) struct Host {
    pub target: Target,
    pub label: String,
    pub state: HostState,
    generation: u64,
    cancel: Arc<AtomicBool>,
}

impl Drop for Host {
    /// Cancel abandoned reads and SSH work. Accepted local file writes finish
    /// even after their Settings window closes, like other local preferences.
    fn drop(&mut self) {
        if matches!(&self.state, HostState::Saving(History::Local(_))) {
            return;
        }
        self.cancel.store(true, Ordering::Release);
    }
}

/// Work for a background worker, fenced by `generation` so a late result
/// cannot overwrite a newer read of the same host.
#[derive(Debug)]
pub(super) struct Job {
    pub target: Target,
    pub generation: u64,
    pub cancel: Arc<AtomicBool>,
}

/// Reads one host's switch; tests swap in a reader that never opens SSH.
pub(super) type HistoryRead =
    Arc<dyn Fn(&Target, &AtomicBool) -> crate::Result<History> + Send + Sync>;

pub(super) struct HostHistory {
    hosts: Vec<Host>,
    generation: u64,
    pub read: HistoryRead,
}

impl Default for HostHistory {
    fn default() -> Self {
        Self {
            hosts: Vec::new(),
            generation: 0,
            read: Arc::new(History::load),
        }
    }
}

impl HostHistory {
    pub fn hosts(&self) -> &[Host] {
        &self.hosts
    }

    fn host(&mut self, target: Target, label: String) -> (Host, Job) {
        self.generation = self.generation.wrapping_add(1);
        let cancel = Arc::new(AtomicBool::new(false));
        let job = Job {
            target: target.clone(),
            generation: self.generation,
            cancel: cancel.clone(),
        };
        let host = Host {
            target,
            label,
            state: HostState::Loading,
            generation: self.generation,
            cancel,
        };
        (host, job)
    }

    /// Follows `connected` (label, target) pairs in order, one entry per
    /// target. New hosts start loading; with `reload`, so do all others
    /// except one mid-save, whose own completion re-reads it.
    pub fn sync(
        &mut self,
        connected: impl IntoIterator<Item = (String, Target)>,
        reload: bool,
    ) -> Vec<Job> {
        let mut previous = std::mem::take(&mut self.hosts);
        let mut jobs = Vec::new();
        for (label, target) in connected {
            if self.hosts.iter().any(|host| host.target == target) {
                continue;
            }
            let kept = previous
                .iter()
                .position(|host| host.target == target)
                .map(|index| previous.swap_remove(index))
                .filter(|host| !reload || matches!(host.state, HostState::Saving(_)));
            let host = match kept {
                Some(mut host) => {
                    host.label = label;
                    host
                }
                None => {
                    let (host, job) = self.host(target, label);
                    jobs.push(job);
                    host
                }
            };
            self.hosts.push(host);
        }
        jobs
    }

    /// Applies a finished read or save. `false` when the host has since gone
    /// or been read again.
    pub fn finish(
        &mut self,
        target: &Target,
        generation: u64,
        result: crate::Result<History>,
    ) -> bool {
        let Some(host) = self
            .hosts
            .iter_mut()
            .find(|host| &host.target == target && host.generation == generation)
        else {
            return false;
        };
        host.state = match result {
            Ok(history) => HostState::Ready(history),
            Err(error) => HostState::Failed(describe(&error)),
        };
        true
    }

    /// Starts a write from a settled read. A host still loading, saving, or
    /// failed has nothing current to compare against.
    pub fn begin_save(&mut self, target: &Target) -> Option<(History, Job)> {
        let index = self.hosts.iter().position(|host| &host.target == target)?;
        let HostState::Ready(history) = &self.hosts[index].state else {
            return None;
        };
        let history = history.clone();
        let label = self.hosts[index].label.clone();
        let (mut host, job) = self.host(target.clone(), label);
        host.state = HostState::Saving(history.clone());
        self.hosts[index] = host;
        Some((history, job))
    }
}

impl SettingsWindow {
    /// The configured local runtime and connected SSH hosts, each with its own file.
    fn connected_hosts(&self, cx: &Context<Self>) -> Vec<(String, Target)> {
        let mut hosts = vec![(
            crate::i18n::fl!("settings-tabs-local-runtime"),
            Target::Local,
        )];
        let Some(source) = self.source.upgrade() else {
            return hosts;
        };
        for endpoint in &source.read(cx).endpoints {
            if let ConnectTarget::Ssh { target, .. } = &endpoint.connection.target
                && endpoint.live.status.is_connected()
                && !hosts
                    .iter()
                    .any(|(_, known)| matches!(known, Target::Ssh(known) if known == target))
            {
                hosts.push((endpoint.label.clone(), Target::Ssh(target.clone())));
            }
        }
        hosts
    }

    /// Cheap when nothing changed: the source notifies on every update, and
    /// this repaints Settings only when its host list does.
    pub(super) fn sync_host_history(&mut self, reload: bool, cx: &mut Context<Self>) {
        let hosts = self.connected_hosts(cx);
        let unchanged = !reload
            && hosts.len() == self.host_history.hosts.len()
            && hosts
                .iter()
                .zip(&self.host_history.hosts)
                .all(|((label, target), host)| *label == host.label && *target == host.target);
        if unchanged {
            return;
        }
        for job in self.host_history.sync(hosts, reload) {
            let read = self.host_history.read.clone();
            let task = cx.background_executor().spawn(async move {
                let result = read(&job.target, &job.cancel);
                (job, result)
            });
            cx.spawn(async move |this, cx| {
                let (job, result) = task.await;
                let _ = this.update(cx, |this, cx| {
                    if this
                        .host_history
                        .finish(&job.target, job.generation, result)
                    {
                        cx.notify();
                    }
                });
            })
            .detach();
        }
        cx.notify();
    }

    pub(super) fn save_host_history(
        &mut self,
        target: &Target,
        enabled: bool,
        cx: &mut Context<Self>,
    ) {
        let Some((history, job)) = self.host_history.begin_save(target) else {
            return;
        };
        let task = cx.background_executor().spawn(async move {
            let result = history.save(enabled, &job.cancel);
            (job, result)
        });
        let source = self.source.clone();
        cx.spawn(async move |this, cx| {
            let (job, result) = task.await;
            // The daemon learns of the edit even when Settings has closed.
            let status = match &result {
                Ok(_) => queue_reload(&source, &job.target, cx),
                Err(_) => None,
            };
            let _ = this.update(cx, |this, cx| {
                if this
                    .host_history
                    .finish(&job.target, job.generation, result)
                {
                    if let Some(status) = status
                        && !this.persistence.saving()
                    {
                        this.status = Some(status);
                    }
                    cx.notify();
                }
            });
        })
        .detach();
        cx.notify();
    }
}

/// The error and its causes. A wrapper that displays its source verbatim
/// would repeat it, so a cause equal to the text before it is skipped.
fn describe(error: &(dyn std::error::Error + 'static)) -> String {
    let mut parts: Vec<String> = Vec::new();
    for cause in std::iter::successors(Some(error), |cause| cause.source()) {
        let text = cause.to_string();
        if parts.last() != Some(&text) && !text.is_empty() {
            parts.push(text);
        }
    }
    parts.join(": ")
}

/// Asks every daemon on `target` to reload its config, returning
/// the first failure to queue that request.
fn queue_reload(
    source: &WeakEntity<crate::SheprdWindow>,
    target: &Target,
    cx: &mut AsyncApp,
) -> Option<String> {
    source
        .update(cx, |source, _| {
            // Every session on the host reads the same config file. Only the
            // shown one reloads now; the others reload when next shown.
            let mut failure = None;
            for endpoint in &mut source.endpoints {
                let matches = match (target, &endpoint.connection.target) {
                    (
                        Target::Local,
                        ConnectTarget::Local
                        | ConnectTarget::Session {
                            development: false, ..
                        },
                    ) => true,
                    (Target::Ssh(host), ConnectTarget::Ssh { target, .. }) => host == target,
                    _ => false,
                };
                if !matches {
                    continue;
                }
                if let Err(error) = endpoint.reload_config() {
                    failure.get_or_insert(error);
                }
            }
            failure.map(|error| {
                crate::i18n::fl!(
                    "settings-remote-history-reload-failed",
                    error = error.localize()
                )
            })
        })
        .ok()
        .flatten()
}

#[cfg(test)]
mod tests;
