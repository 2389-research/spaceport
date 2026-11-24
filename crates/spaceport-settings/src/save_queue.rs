//! The bounded queue behind Settings' saves: what a running write may leave
//! pending, and the transitions that drain it.
//!
//! The queue owns only *when* a write may start and what waits behind it; the
//! write itself, its reload and its failures belong to the caller, so the
//! queued item is the caller's type. A queued write is told when a finished
//! one moved the shared file on, which is the only thing the queue needs of it.

use crate::herdr_settings::{Edit, Settings};
use crate::sidebar_draft::SidebarIntent;
use crate::{LayoutMode, Result};
use std::collections::VecDeque;
use std::sync::Arc;

/// How many writes may wait behind a running one. A burst of accepted edits
/// past this is refused rather than grown without bound.
pub const QUEUE_LIMIT: usize = 64;

#[derive(Default)]
enum Save {
    // keep-sorted start
    #[default]
    Idle,
    Loading,
    Running(Kind),
    // keep-sorted end
}

/// What a running write is saving, so its completion can tell a stale draft
/// from the one it wrote.
pub enum Kind {
    // keep-sorted start
    Layout(LayoutMode),
    Ordinary,
    Sidebar(SidebarIntent),
    Theme(u64),
    // keep-sorted end
}

#[derive(Default)]
enum Goal {
    // keep-sorted start
    Closing,
    #[default]
    Editing,
    Flushing,
    Quitting,
    // keep-sorted end
}

enum Event<Q> {
    // keep-sorted start
    AbortDrain,
    AdvanceShared(SharedWrite),
    BeginLoad,
    BeginSave(Kind),
    Close,
    FinishLoad,
    FinishSave,
    Flush,
    Next,
    Queue(Q),
    QueueSidebar(Option<SidebarIntent>),
    Quit,
    SaveFailed,
    StopFlush,
    TakeClose,
    TakePending,
    TakeSidebar,
    // keep-sorted end
}

enum Effect<Q> {
    // keep-sorted start
    Closed(bool),
    Finished(Kind),
    Next(Option<Q>),
    None,
    Pending(VecDeque<Q>),
    Queued(bool),
    Sidebar(Option<SidebarIntent>),
    // keep-sorted end
}

/// A write waiting behind the running one, which a finished shared write
/// leaves pointing at the file it wrote.
pub trait PendingWrite {
    fn advance_shared(&mut self, write: &SharedWrite);
}

pub struct Lifecycle<Q> {
    // keep-sorted start
    goal: Goal,
    pending: VecDeque<Q>,
    save: Save,
    sidebar_pending: Option<SidebarIntent>,
    // keep-sorted end
}

impl<Q> Default for Lifecycle<Q> {
    fn default() -> Self {
        Self {
            goal: Goal::default(),
            pending: VecDeque::new(),
            save: Save::default(),
            sidebar_pending: None,
        }
    }
}

impl<Q: PendingWrite> Lifecycle<Q> {
    /// The only writer of persistence state. Quit preserves the running kind
    /// so shutdown can compare its captured intent with the latest draft.
    fn transition(&mut self, event: Event<Q>) -> Effect<Q> {
        match event {
            Event::AdvanceShared(write) => {
                for pending in &mut self.pending {
                    pending.advance_shared(&write);
                }
            }
            Event::BeginLoad => {
                debug_assert!(!self.busy());
                self.save = Save::Loading;
            }
            Event::FinishLoad => {
                debug_assert!(self.loading());
                self.save = Save::Idle;
            }
            Event::BeginSave(kind) => {
                debug_assert!(!self.busy());
                self.save = Save::Running(kind);
            }
            Event::FinishSave => {
                return match std::mem::take(&mut self.save) {
                    Save::Running(kind) => Effect::Finished(kind),
                    _ => unreachable!("only a running write completes"),
                };
            }
            Event::Flush if !self.busy() && !self.closing() => self.goal = Goal::Flushing,
            Event::StopFlush if self.flushing() => self.goal = Goal::Editing,
            Event::Close if !self.quitting() => self.goal = Goal::Closing,
            Event::TakeClose => {
                let closing = self.closing();
                if closing {
                    self.goal = Goal::Editing;
                }
                return Effect::Closed(closing);
            }
            Event::Quit => self.goal = Goal::Quitting,
            event @ (Event::AbortDrain | Event::SaveFailed) => {
                if !self.quitting() {
                    self.goal = Goal::Editing;
                }
                if matches!(event, Event::SaveFailed) {
                    self.sidebar_pending = None;
                }
            }
            Event::QueueSidebar(intent) => self.sidebar_pending = intent,
            Event::Queue(request) => {
                let accepted =
                    !self.quitting() && !self.closing() && self.pending.len() < QUEUE_LIMIT;
                if accepted {
                    self.pending.push_back(request);
                }
                return Effect::Queued(accepted);
            }
            Event::Next => {
                return Effect::Next(if self.busy() {
                    None
                } else {
                    self.pending.pop_front()
                });
            }
            Event::TakePending => return Effect::Pending(std::mem::take(&mut self.pending)),
            Event::TakeSidebar => {
                return Effect::Sidebar(if self.busy() {
                    None
                } else {
                    self.sidebar_pending.take()
                });
            }
            Event::Flush | Event::StopFlush | Event::Close => {}
        }
        Effect::None
    }

    pub fn loading(&self) -> bool {
        matches!(self.save, Save::Loading)
    }
    pub fn saving(&self) -> bool {
        matches!(self.save, Save::Running(_))
    }
    pub fn quitting(&self) -> bool {
        matches!(self.goal, Goal::Quitting)
    }
    pub fn busy(&self) -> bool {
        !matches!(self.save, Save::Idle) || self.quitting()
    }
    pub fn flushing(&self) -> bool {
        matches!(self.goal, Goal::Flushing)
    }
    /// The user asked to close the window while writes drained; the caller
    /// keeps which window that was.
    pub fn closing(&self) -> bool {
        matches!(self.goal, Goal::Closing)
    }
    pub fn kind(&self) -> Option<&Kind> {
        match &self.save {
            Save::Running(kind) => Some(kind),
            _ => None,
        }
    }
    pub fn sidebar_queued(&self) -> bool {
        self.sidebar_pending.is_some()
    }

    pub fn flush(&mut self) {
        self.transition(Event::Flush);
    }
    pub fn stop_flush(&mut self) {
        self.transition(Event::StopFlush);
    }
    pub fn close(&mut self) {
        self.transition(Event::Close);
    }
    pub fn quit(&mut self) {
        self.transition(Event::Quit);
    }
    pub fn advance_shared(&mut self, write: SharedWrite) {
        self.transition(Event::AdvanceShared(write));
    }
    pub fn abort_drain(&mut self) {
        self.transition(Event::AbortDrain);
    }
    pub fn fail(&mut self) {
        self.transition(Event::SaveFailed);
    }
    pub fn begin_load(&mut self) {
        self.transition(Event::BeginLoad);
    }
    pub fn finish_load(&mut self) {
        self.transition(Event::FinishLoad);
    }
    pub fn begin_save(&mut self, kind: Kind) {
        self.transition(Event::BeginSave(kind));
    }
    pub fn queue_sidebar(&mut self, intent: Option<SidebarIntent>) {
        self.transition(Event::QueueSidebar(intent));
    }
    pub fn take_sidebar(&mut self) -> Option<SidebarIntent> {
        let Effect::Sidebar(intent) = self.transition(Event::TakeSidebar) else {
            unreachable!("sidebar transition returns its committed intent")
        };
        intent
    }

    /// `true` when the close the window asked for is the one taken.
    pub fn take_close(&mut self) -> bool {
        let Effect::Closed(closing) = self.transition(Event::TakeClose) else {
            unreachable!("close transition returns its window")
        };
        closing
    }
    pub fn finish_save(&mut self) -> Kind {
        let Effect::Finished(kind) = self.transition(Event::FinishSave) else {
            unreachable!("save transition returns its kind")
        };
        kind
    }
    pub fn enqueue(&mut self, request: Q) -> bool {
        let Effect::Queued(accepted) = self.transition(Event::Queue(request)) else {
            unreachable!("queue transition reports acceptance")
        };
        accepted
    }
    pub fn take_next(&mut self) -> Option<Q> {
        let Effect::Next(request) = self.transition(Event::Next) else {
            unreachable!("next transition returns its request")
        };
        request
    }
    pub fn take_pending(&mut self) -> VecDeque<Q> {
        let Effect::Pending(pending) = self.transition(Event::TakePending) else {
            unreachable!("quit transition returns pending writes")
        };
        pending
    }
}

/// The reload a write runs once it has landed, which reconciles it.
pub type Load<L, E> = Box<dyn FnOnce() -> Result<L, E> + Send>;

/// One write and the reload that reconciles it, queued or running.
pub struct Request<L, E> {
    // keep-sorted start
    load: Load<L, E>,
    operation: Operation<E>,
    shared: bool,
    // keep-sorted end
}

pub enum Operation<E> {
    // keep-sorted start
    Captured(Box<dyn FnOnce() -> Result<Option<SharedWrite>, E> + Send>),
    Shared { snapshot: Arc<Settings>, edit: Edit },
    // keep-sorted end
}

#[derive(Clone)]
pub struct SharedWrite {
    before: Arc<Settings>,
    after: Arc<Settings>,
}

impl SharedWrite {
    pub fn save(before: Settings, edit: Edit) -> Result<Self> {
        Self::save_expected(Arc::new(before), edit)
    }

    fn save_expected(before: Arc<Settings>, edit: Edit) -> Result<Self> {
        let after = Arc::new(before.save(edit)?);
        Ok(Self { before, after })
    }
}

impl<E> Operation<E> {
    pub fn plain(operation: impl FnOnce() -> Result<(), E> + Send + 'static) -> Self {
        Self::Captured(Box::new(move || operation().map(|()| None)))
    }

    /// Runs the write. A shared edit's failure is this crate's own, so the
    /// caller's error must carry it.
    pub fn run(self) -> Result<Option<SharedWrite>, E>
    where
        E: From<crate::Error>,
    {
        match self {
            Self::Captured(operation) => operation(),
            Self::Shared { snapshot, edit } => SharedWrite::save_expected(snapshot, edit)
                .map(Some)
                .map_err(E::from),
        }
    }
}

impl<L, E> Request<L, E> {
    pub fn new(operation: Operation<E>, load: Load<L, E>, shared: bool) -> Self {
        Self {
            load,
            operation,
            shared,
        }
    }

    pub fn into_parts(self) -> (Operation<E>, Load<L, E>, bool) {
        (self.operation, self.load, self.shared)
    }
}

impl<L, E> PendingWrite for Request<L, E> {
    fn advance_shared(&mut self, write: &SharedWrite) {
        if let Operation::Shared { snapshot, .. } = &mut self.operation
            && (Arc::ptr_eq(snapshot, &write.before) || snapshot.same_snapshot(&write.before))
        {
            *snapshot = write.after.clone();
        }
    }
}

/// What a finished write left the caller holding: the shared settings its
/// reload read, and the write itself when that reload reflects it.
#[derive(Clone, Default)]
pub struct Reconciled {
    pub snapshot: Option<Settings>,
    pub write: Option<SharedWrite>,
}

/// Ties a queued shared edit to the snapshot it expects, and gives up when
/// the reload does not show this write, or shows a later one done elsewhere.
pub fn reconcile<E>(
    saved: &Result<Option<SharedWrite>, E>,
    shared: Option<&Settings>,
) -> Reconciled {
    if saved.is_err() {
        return Reconciled::default();
    }
    let snapshot = shared.cloned();
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
