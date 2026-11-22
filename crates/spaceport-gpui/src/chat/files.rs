// ABOUTME: Owns pending file operations and their conversation fences.
// ABOUTME: Attaches completed files only to their original conversation.
use super::{Target, chats::ChatPane};
use crate::{
    SpaceportWindow,
    i18n::{Localize, fl},
};
use gpui::{Context, PathPromptOptions, Window};
use spaceport_chat_protocol::{Cursor, MAX_ATTACHMENTS};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// A callback's identity, conversation, and worker cancellation signal.
#[derive(Clone)]
pub(super) struct Operation {
    cancelled: Arc<AtomicBool>,
    count: usize,
    cursor: Option<Cursor>,
    kind: Kind,
    target: Target,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    // keep-sorted start
    Prepare,
    Upload,
    // keep-sorted end
}

impl Operation {
    pub(super) fn cancelled(&self) -> Arc<AtomicBool> {
        self.cancelled.clone()
    }
}

/// The pane owns all unfinished work, including dialogs that prepare no files yet.
#[derive(Default)]
enum State {
    // keep-sorted start
    #[default]
    Idle,
    Pending(Vec<Operation>),
    // keep-sorted end
}

#[derive(Default)]
pub(super) struct FileWork {
    state: State,
}

enum Event<'a> {
    // keep-sorted start
    Begin(Operation),
    Cancel,
    Finish {
        target: &'a Target,
        cursor: Option<Cursor>,
        operation: &'a Operation,
    },
    Synchronize {
        target: &'a Target,
        cursor: Option<Cursor>,
    },
    // keep-sorted end
}

impl FileWork {
    pub(super) fn begin(
        &mut self,
        target: &Target,
        cursor: Option<Cursor>,
        count: usize,
    ) -> Operation {
        self.begin_kind(target, cursor, count, Kind::Prepare)
    }

    fn begin_kind(
        &mut self,
        target: &Target,
        cursor: Option<Cursor>,
        count: usize,
        kind: Kind,
    ) -> Operation {
        let operation = Operation {
            cancelled: Arc::new(AtomicBool::new(false)),
            count,
            cursor,
            kind,
            target: target.clone(),
        };
        self.transition(Event::Begin(operation.clone()));
        operation
    }

    pub(super) fn pending(&self, target: &Target, cursor: Option<Cursor>) -> usize {
        match &self.state {
            State::Idle => 0,
            State::Pending(operations) => operations
                .iter()
                .filter(|operation| is_current(target, cursor, &operation.target, operation.cursor))
                .map(|operation| operation.count)
                .sum(),
        }
    }

    fn copying(&self, target: &Target, cursor: Option<Cursor>) -> bool {
        match &self.state {
            State::Idle => false,
            State::Pending(operations) => operations.iter().any(|operation| {
                operation.kind == Kind::Upload
                    && is_current(target, cursor, &operation.target, operation.cursor)
            }),
        }
    }

    pub(super) fn synchronize(&mut self, target: &Target, cursor: Option<Cursor>) {
        self.transition(Event::Synchronize { target, cursor });
    }

    pub(super) fn finish(
        &mut self,
        target: &Target,
        cursor: Option<Cursor>,
        operation: &Operation,
    ) -> bool {
        self.transition(Event::Finish {
            target,
            cursor,
            operation,
        })
    }

    /// The sole writer of lifecycle state and worker cancellation signals.
    fn transition(&mut self, event: Event<'_>) -> bool {
        let conversation = match &event {
            Event::Begin(operation) => Some((&operation.target, operation.cursor)),
            Event::Finish { target, cursor, .. } | Event::Synchronize { target, cursor } => {
                Some((*target, *cursor))
            }
            Event::Cancel => None,
        };
        if let State::Pending(operations) = &mut self.state {
            operations.retain(|operation| {
                let current = conversation.is_some_and(|(target, cursor)| {
                    is_current(target, cursor, &operation.target, operation.cursor)
                });
                if !current {
                    operation.cancelled.store(true, Ordering::Release);
                }
                current
            });
        }
        let accepted = match event {
            Event::Begin(operation) => {
                match &mut self.state {
                    State::Idle => self.state = State::Pending(vec![operation]),
                    State::Pending(operations) => operations.push(operation),
                }
                true
            }
            Event::Finish { operation, .. } => {
                if let State::Pending(operations) = &mut self.state
                    && let Some(index) = operations
                        .iter()
                        .position(|pending| Arc::ptr_eq(&pending.cancelled, &operation.cancelled))
                {
                    operations.remove(index);
                    true
                } else {
                    false
                }
            }
            Event::Synchronize { .. } | Event::Cancel => false,
        };
        if matches!(&self.state, State::Pending(operations) if operations.is_empty()) {
            self.state = State::Idle;
        }
        accepted
    }
}

impl Drop for FileWork {
    fn drop(&mut self) {
        self.transition(Event::Cancel);
    }
}

pub(super) fn is_current(
    target: &Target,
    cursor: Option<Cursor>,
    captured_target: &Target,
    captured_cursor: Option<Cursor>,
) -> bool {
    target == captured_target
        && cursor.map(|cursor| cursor.epoch) == captured_cursor.map(|cursor| cursor.epoch)
}

impl ChatPane {
    pub(crate) fn uploading(&self) -> usize {
        self.file_work.pending(&self.target, self.session.since())
    }

    pub(super) fn begin_files(&mut self, count: usize) -> Operation {
        self.file_work
            .begin(&self.target, self.session.since(), count)
    }

    pub(super) fn finish_files(&mut self, operation: &Operation) -> bool {
        self.file_work
            .finish(&self.target, self.session.since(), operation)
    }
}

impl SpaceportWindow {
    /// Copies files off the UI thread; stale successes are cleaned on the original host.
    pub(super) fn upload_chat_files(
        &mut self,
        pane_id: &str,
        host: String,
        mut paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(pane) = self.chats.pane_mut(pane_id) else {
            return;
        };
        if paths.is_empty() {
            return;
        }
        if pane.file_work.copying(&pane.target, pane.session.since()) {
            pane.error = Some(fl!("chat-files-copying"));
            self.redraw_terminal(cx);
            cx.notify();
            return;
        }
        let available = MAX_ATTACHMENTS.saturating_sub(pane.attachments.len());
        if paths.len() > available {
            pane.error = Some(fl!("chat-files-too-many", count = MAX_ATTACHMENTS));
            paths.truncate(available);
        }
        if paths.is_empty() {
            self.redraw_terminal(cx);
            cx.notify();
            return;
        }
        let operation = pane.file_work.begin_kind(
            &pane.target,
            pane.session.since(),
            paths.len(),
            Kind::Upload,
        );
        let cancelled = operation.cancelled();
        let executor = cx.background_executor().clone();
        let worker_host = host.clone();
        let upload = executor.spawn(async move {
            spaceport_client::upload_files(&worker_host, &paths, &cancelled, |_, _| {})
        });
        let pane_id = pane_id.to_owned();
        // Keep awaiting after the weak entity or window disappears, so unclaimed copies are removed.
        cx.spawn_in(window, async move |this, cx| {
            let uploaded = upload.await;
            let accepted = this
                .update_in(cx, |this, window, cx| {
                    this.finish_chat_upload(&pane_id, &operation, &uploaded, window, cx)
                })
                .unwrap_or(false);
            if !accepted && let Ok(paths) = uploaded {
                let cleanup = executor
                    .spawn(async move { spaceport_client::remove_uploaded_files(&host, &paths) })
                    .await;
                if let Err(error) = cleanup {
                    // i18n: data
                    crate::storage_warning::warn_storage_failure("Chat upload cleanup", &error);
                    tracing::warn!(%error, "Unclaimed chat upload cleanup failed");
                }
            }
        })
        .detach();
        self.redraw_terminal(cx);
    }

    pub(super) fn finish_chat_upload(
        &mut self,
        pane_id: &str,
        operation: &Operation,
        uploaded: &spaceport_client::Result<Vec<String>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(pane) = self.chats.pane_mut(pane_id) else {
            return false;
        };
        if !pane.finish_files(operation) {
            return false;
        }
        if pane.input() != super::chats::ChatInput::Composer {
            self.redraw_terminal(cx);
            return false;
        }
        match uploaded {
            Ok(remote) => {
                // Reject the whole batch if capacity changed, so the caller
                // cleans every copy instead of leaking a discarded suffix.
                if remote.len() > MAX_ATTACHMENTS.saturating_sub(pane.attachments.len()) {
                    pane.error = Some(fl!("chat-files-too-many", count = MAX_ATTACHMENTS));
                    self.redraw_terminal(cx);
                    cx.notify();
                    return false;
                }
                self.add_chat_files(
                    pane_id,
                    remote.iter().map(PathBuf::from).collect(),
                    window,
                    cx,
                );
                true
            }
            Err(error) => {
                pane.error = Some(fl!("chat-files-not-attached", error = error.localize()));
                self.redraw_terminal(cx);
                false
            }
        }
    }

    /// Adds paths, as the program will read them, to the next prompt,
    /// skipping ones already there and any past [`MAX_ATTACHMENTS`]. Whether
    /// each exists is checked by the program when the prompt goes out.
    pub(super) fn add_chat_files(
        &mut self,
        pane_id: &str,
        paths: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(pane) = self.chats.pane_mut(pane_id) else {
            return;
        };
        if pane.input() != super::chats::ChatInput::Composer {
            return;
        }
        for path in paths {
            if pane.attachments.len() >= MAX_ATTACHMENTS {
                pane.error = Some(fl!("chat-files-too-many", count = MAX_ATTACHMENTS));
                break;
            }
            if !pane.attachments.contains(&path) {
                pane.attachments.push(path);
            }
        }
        if let Some(input) = pane.composer.clone() {
            let focus = input.read(cx).focus.clone();
            window.focus(&focus, cx);
        }
        self.redraw_terminal(cx);
        cx.notify();
    }

    pub(super) fn browse_chat_files(
        &mut self,
        pane_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(pane) = self.chats.pane_mut(pane_id) else {
            return;
        };
        let operation = pane.begin_files(0);
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some(fl!("chat-files-attach").into()),
        });
        let pane_id = pane_id.to_owned();
        cx.spawn_in(window, async move |this, cx| {
            let chosen = paths.await;
            let _ = this.update_in(cx, |this, window, cx| {
                let Some(pane) = this.chats.pane_mut(&pane_id) else {
                    return;
                };
                if !pane.finish_files(&operation) {
                    return;
                }
                if let Ok(Ok(Some(paths))) = chosen {
                    this.attach_chat_files(&pane_id, paths, window, cx);
                }
            });
        })
        .detach();
    }
}

#[cfg(test)]
mod tests;
