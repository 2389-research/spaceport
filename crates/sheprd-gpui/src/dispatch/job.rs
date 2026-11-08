//! A worktree or workspace being created on another host. It runs on its own named thread and
//! reports through a channel the window drains on its tick. Closing the
//! dialog does not stop it, since stopping halfway could leave a clone or a
//! stray reference behind; only closing the window does.

use crate::teleport::{self, Created, FreshOrigin, HostRepositories, Step};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

enum Event {
    Step(Step),
    Done(crate::Result<Created>),
}

/// What a dispatch makes once its destination is ready.
pub(crate) enum Then {
    Workspace { label: Option<String> },
}

impl Then {
    fn run(
        &self,
        origin: &FreshOrigin,
        destination: &HostRepositories,
        report: &mut dyn FnMut(Step),
        cancelled: &AtomicBool,
    ) -> crate::Result<Created> {
        Ok(match self {
            Self::Workspace { label } => teleport::dispatch_workspace(
                origin,
                destination,
                label.as_deref(),
                report,
                cancelled,
            )?,
        })
    }
}

pub(crate) struct Job {
    /// The destination's name, for the dialog and the flash.
    pub(crate) host: String,
    /// The repository's name, recorded in the history once the job succeeds.
    pub(crate) repo: String,
    step: Option<Step>,
    events: mpsc::Receiver<Event>,
    cancelled: Arc<AtomicBool>,
}

impl Drop for Job {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
    }
}

impl Job {
    fn spawn(
        host: String,
        repo: String,
        work: impl FnOnce(&mpsc::Sender<Event>, &AtomicBool) -> crate::Result<Created> + Send + 'static,
    ) -> Self {
        let (sender, events) = mpsc::channel();
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = cancelled.clone();
        let spawned = std::thread::Builder::new()
            .name("herdr-dispatch".into())
            .spawn(move || {
                let done = work(&sender, &flag);
                let _ = sender.send(Event::Done(done));
            });
        if let Err(error) = spawned {
            tracing::warn!(%error, "could not start the dispatch worker");
        }
        Self {
            host,
            repo,
            step: None,
            events,
            cancelled,
        }
    }

    /// Make `then` on `destination`.
    fn on_host(origin: FreshOrigin, destination: HostRepositories, then: Then) -> Self {
        Self::spawn(
            destination.place.label.clone(),
            origin.repo_label.clone(),
            move |events, cancelled| {
                let mut report = |step| {
                    let _ = events.send(Event::Step(step));
                };
                then.run(&origin, &destination, &mut report, cancelled)
            },
        )
    }

    /// Open `origin`'s repository as a new workspace on `destination`.
    pub(crate) fn workspace(
        origin: FreshOrigin,
        destination: HostRepositories,
        label: Option<String>,
    ) -> Self {
        Self::on_host(origin, destination, Then::Workspace { label })
    }

    /// Take reported steps. Returns whether the status changed, and the
    /// outcome once, with its failure as display text.
    pub(crate) fn poll(&mut self) -> (bool, Option<Result<Created, String>>) {
        let mut changed = false;
        while let Ok(event) = self.events.try_recv() {
            changed = true;
            match event {
                Event::Step(step) => self.step = Some(step),
                Event::Done(result) => {
                    return (
                        true,
                        Some(result.map_err(|error| {
                            tracing::warn!(%error, host = %self.host, "dispatch");
                            crate::i18n::Localize::localize(&error)
                        })),
                    );
                }
            }
        }
        (changed, None)
    }

    /// What the job is doing, for the dialog.
    pub(crate) fn status(&self) -> String {
        match self.step {
            None => crate::i18n::fl!("dispatch-starting", host = self.host.as_str()),
            Some(step) => crate::i18n::fl!(
                "dispatch-step",
                host = self.host.as_str(),
                step = teleport::text::step(Some(step))
            ),
        }
    }
}
