// ABOUTME: Owns Settings persistence transitions and bounded pending writes.
// ABOUTME: Separates active I/O from the user's flush, close, or quit goal.
use super::super::sidebar::SidebarIntent;
use crate::config::LayoutMode;
use gpui::AnyWindowHandle;
use std::collections::VecDeque;

#[derive(Default)]
enum Save {
    // keep-sorted start
    #[default]
    Idle,
    Loading,
    Running(Kind),
    // keep-sorted end
}

pub(in crate::settings_window) enum Kind {
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
    Closing(AnyWindowHandle),
    #[default]
    Editing,
    Flushing,
    Quitting,
    // keep-sorted end
}

enum Event {
    // keep-sorted start
    AbortDrain,
    AdvanceShared(super::SharedWrite),
    BeginLoad,
    BeginSave(Kind),
    Close(AnyWindowHandle),
    FinishLoad,
    FinishSave,
    Flush,
    Next,
    Queue(super::Request),
    QueueSidebar(Option<SidebarIntent>),
    Quit,
    SaveFailed,
    StopFlush,
    TakeClose,
    TakePending,
    TakeSidebar,
    // keep-sorted end
}

enum Effect {
    // keep-sorted start
    Closed(Option<AnyWindowHandle>),
    Finished(Kind),
    Next(Option<super::Request>),
    None,
    Pending(VecDeque<super::Request>),
    Queued(bool),
    Sidebar(Option<SidebarIntent>),
    // keep-sorted end
}

#[derive(Default)]
pub(in crate::settings_window) struct Lifecycle {
    // keep-sorted start
    goal: Goal,
    pending: VecDeque<super::Request>,
    save: Save,
    sidebar_pending: Option<SidebarIntent>,
    // keep-sorted end
}

pub(in crate::settings_window) const QUEUE_LIMIT: usize = 64;

impl Lifecycle {
    /// The only writer of persistence state. Quit preserves the running kind
    /// so shutdown can compare its captured intent with the latest draft.
    fn transition(&mut self, event: Event) -> Effect {
        match event {
            Event::AdvanceShared(write) => {
                for request in &mut self.pending {
                    request.advance_shared(&write);
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
            Event::Flush if !self.busy() && self.closing().is_none() => self.goal = Goal::Flushing,
            Event::StopFlush if self.flushing() => self.goal = Goal::Editing,
            Event::Close(handle) if !self.quitting() => self.goal = Goal::Closing(handle),
            Event::TakeClose => {
                let handle = self.closing();
                if handle.is_some() {
                    self.goal = Goal::Editing;
                }
                return Effect::Closed(handle);
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
                let accepted = !self.quitting()
                    && self.closing().is_none()
                    && self.pending.len() < QUEUE_LIMIT;
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
            Event::Flush | Event::StopFlush | Event::Close(_) => {}
        }
        Effect::None
    }

    pub(in crate::settings_window) fn loading(&self) -> bool {
        matches!(self.save, Save::Loading)
    }
    pub(in crate::settings_window) fn saving(&self) -> bool {
        matches!(self.save, Save::Running(_))
    }
    pub(in crate::settings_window) fn quitting(&self) -> bool {
        matches!(self.goal, Goal::Quitting)
    }
    pub(in crate::settings_window) fn busy(&self) -> bool {
        !matches!(self.save, Save::Idle) || self.quitting()
    }
    pub(in crate::settings_window) fn flushing(&self) -> bool {
        matches!(self.goal, Goal::Flushing)
    }
    pub(in crate::settings_window) fn closing(&self) -> Option<AnyWindowHandle> {
        match self.goal {
            Goal::Closing(handle) => Some(handle),
            _ => None,
        }
    }
    pub(in crate::settings_window) fn kind(&self) -> Option<&Kind> {
        match &self.save {
            Save::Running(kind) => Some(kind),
            _ => None,
        }
    }
    pub(in crate::settings_window) fn sidebar_queued(&self) -> bool {
        self.sidebar_pending.is_some()
    }

    pub(in crate::settings_window) fn flush(&mut self) {
        self.transition(Event::Flush);
    }
    pub(in crate::settings_window) fn stop_flush(&mut self) {
        self.transition(Event::StopFlush);
    }
    pub(in crate::settings_window) fn close(&mut self, handle: AnyWindowHandle) {
        self.transition(Event::Close(handle));
    }
    pub(in crate::settings_window) fn quit(&mut self) {
        self.transition(Event::Quit);
    }
    pub(in crate::settings_window) fn advance_shared(&mut self, write: super::SharedWrite) {
        self.transition(Event::AdvanceShared(write));
    }
    pub(in crate::settings_window) fn abort_drain(&mut self) {
        self.transition(Event::AbortDrain);
    }
    pub(in crate::settings_window) fn fail(&mut self) {
        self.transition(Event::SaveFailed);
    }
    pub(in crate::settings_window) fn begin_load(&mut self) {
        self.transition(Event::BeginLoad);
    }
    pub(in crate::settings_window) fn finish_load(&mut self) {
        self.transition(Event::FinishLoad);
    }
    pub(in crate::settings_window) fn begin_save(&mut self, kind: Kind) {
        self.transition(Event::BeginSave(kind));
    }
    pub(in crate::settings_window) fn queue_sidebar(&mut self, intent: Option<SidebarIntent>) {
        self.transition(Event::QueueSidebar(intent));
    }
    pub(in crate::settings_window) fn take_sidebar(&mut self) -> Option<SidebarIntent> {
        let Effect::Sidebar(intent) = self.transition(Event::TakeSidebar) else {
            unreachable!("sidebar transition returns its committed intent")
        };
        intent
    }

    pub(in crate::settings_window) fn take_close(&mut self) -> Option<AnyWindowHandle> {
        let Effect::Closed(handle) = self.transition(Event::TakeClose) else {
            unreachable!("close transition returns its window")
        };
        handle
    }
    pub(in crate::settings_window) fn finish_save(&mut self) -> Kind {
        let Effect::Finished(kind) = self.transition(Event::FinishSave) else {
            unreachable!("save transition returns its kind")
        };
        kind
    }
    pub(in crate::settings_window) fn enqueue(&mut self, request: super::Request) -> bool {
        let Effect::Queued(accepted) = self.transition(Event::Queue(request)) else {
            unreachable!("queue transition reports acceptance")
        };
        accepted
    }
    pub(in crate::settings_window) fn next(&mut self) -> Option<super::Request> {
        let Effect::Next(request) = self.transition(Event::Next) else {
            unreachable!("next transition returns its request")
        };
        request
    }
    pub(in crate::settings_window) fn take_pending(&mut self) -> VecDeque<super::Request> {
        let Effect::Pending(pending) = self.transition(Event::TakePending) else {
            unreachable!("quit transition returns pending writes")
        };
        pending
    }
}
