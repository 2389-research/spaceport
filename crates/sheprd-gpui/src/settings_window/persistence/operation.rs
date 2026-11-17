// ABOUTME: Keeps queued shared edits tied to their expected file snapshots.
// ABOUTME: Advances them only through a successfully reconciled owned replacement.
use super::Loaded;
use crate::herdr_settings::{Edit, Settings};
use std::sync::Arc;

pub(in crate::settings_window) struct Request {
    pub(super) operation: Operation,
    pub(super) load: Box<dyn FnOnce() -> crate::Result<Loaded> + Send>,
    pub(super) shared: bool,
}

pub(super) enum Operation {
    // keep-sorted start
    Captured(Box<dyn FnOnce() -> crate::Result<Option<SharedWrite>> + Send>),
    Shared { snapshot: Arc<Settings>, edit: Edit },
    // keep-sorted end
}

#[derive(Clone)]
pub(in crate::settings_window) struct SharedWrite {
    before: Arc<Settings>,
    after: Arc<Settings>,
}

impl SharedWrite {
    pub(in crate::settings_window) fn save(before: Settings, edit: Edit) -> crate::Result<Self> {
        Self::save_expected(Arc::new(before), edit)
    }

    fn save_expected(before: Arc<Settings>, edit: Edit) -> crate::Result<Self> {
        let after = Arc::new(before.save(edit)?);
        Ok(Self { before, after })
    }
}

impl Operation {
    pub(super) fn plain(operation: impl FnOnce() -> crate::Result<()> + Send + 'static) -> Self {
        Self::Captured(Box::new(move || operation().map(|()| None)))
    }

    pub(super) fn run(self) -> crate::Result<Option<SharedWrite>> {
        match self {
            Self::Captured(operation) => operation(),
            Self::Shared { snapshot, edit } => SharedWrite::save_expected(snapshot, edit).map(Some),
        }
    }
}

impl Request {
    pub(super) fn advance_shared(&mut self, write: &SharedWrite) {
        if let Operation::Shared { snapshot, .. } = &mut self.operation
            && (Arc::ptr_eq(snapshot, &write.before) || snapshot.same_snapshot(&write.before))
        {
            *snapshot = write.after.clone();
        }
    }
}

#[derive(Clone, Default)]
pub(in crate::settings_window) struct Reconciled {
    pub(super) snapshot: Option<Settings>,
    pub(super) write: Option<SharedWrite>,
}

pub(super) fn reconcile(
    saved: &crate::Result<Option<SharedWrite>>,
    loaded: &crate::Result<Loaded>,
) -> Reconciled {
    if saved.is_err() {
        return Reconciled::default();
    }
    let snapshot = loaded
        .as_ref()
        .ok()
        .and_then(|loaded| loaded.shared.clone());
    if let Ok(Some(write)) = saved {
        // A successful reload may already reflect an intervening external writer.
        // Never promote that snapshot into an edit accepted against our old file.
        if !snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.same_snapshot(&write.after))
        {
            return Reconciled::default();
        }
        return Reconciled {
            snapshot,
            write: Some(write.clone()),
        };
    }
    Reconciled {
        snapshot,
        write: None,
    }
}
