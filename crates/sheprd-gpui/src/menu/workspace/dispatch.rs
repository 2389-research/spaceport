//! Submitting a new workspace dialog whose host picker chose
//! another host: the work runs as a dispatch job instead of
//! a daemon request.

use super::chosen_label;
use crate::{
    SheprdWindow,
    dispatch::{Destination, Job, Target},
    menu::WorkspaceAction,
};

impl SheprdWindow {
    /// Where the new workspace dialog's picker sends its request.
    pub(in crate::menu) fn dispatch_choice(&self, action: WorkspaceAction) -> Option<Target> {
        if action != WorkspaceAction::NewWorkspace {
            return None;
        }
        match self.menu.dispatch.as_ref()?.destination() {
            Destination::Here => None,
            Destination::Host(host) => Some(Target::Host(host.endpoint_id.clone())),
        }
    }

    /// Start creating on `target`. The dialog stays open on the job's
    /// progress, and the window follows the result once it lands.
    pub(in crate::menu) fn submit_dispatch(&mut self, target: &Target) -> crate::Result<()> {
        let Target::Host(endpoint_id) = target;
        let endpoint_id = endpoint_id.as_str();
        if self.dispatch_job.is_some() {
            return Err(crate::dispatch::Error::Busy.into());
        }
        let target = self
            .menu
            .target
            .as_ref()
            .ok_or(crate::Error::StaleWorkspace)?;
        let tree = target
            .worktree
            .as_ref()
            .ok_or(crate::Error::WorkspaceRepositoryChanged)?;
        let destination = self.dispatch_destination(endpoint_id);
        let origin = self.dispatch_origin(&target.id, &tree.key, &tree.label);
        let (Some(origin), Some(destination)) = (origin, destination) else {
            let label = self
                .endpoints
                .iter()
                .find(|endpoint| endpoint.id == endpoint_id)
                .map_or_else(|| endpoint_id.to_owned(), |e| e.label.clone());
            return Err(crate::dispatch::Error::HostUnavailable(label).into());
        };
        let label = self.dispatch_label();
        let job = Job::workspace(origin, destination, label);
        self.dispatch_job = Some(job);
        self.menu.error = None;
        Ok(())
    }

    /// What the open dialog asks for, from its fields.
    fn dispatch_label(&self) -> Option<String> {
        let text = self
            .menu
            .input
            .as_ref()
            .map_or("", |input| input.text.as_str())
            .trim();
        chosen_label(text, self.menu.suggested_name.as_deref()).map(str::to_owned)
    }
}
