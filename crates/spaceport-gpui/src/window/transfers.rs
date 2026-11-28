// ABOUTME: Uploads files off the UI thread and pastes into their captured terminal.
// ABOUTME: Owns transfer transitions, readiness waits, cancellation, and cleanup.
//! Large files travel on an independent SSH stream; the daemon input lane stays
//! responsive. Only completed copies can paste paths, into their captured target.
// i18n: migrated

use super::{SpaceportWindow, pending_input::Delivery};
use crate::i18n::{Localize, fl};
use crate::{
    connection::ConnectionBridge, endpoint::fence::InputFence, fonts::StyledFont,
    terminal::InputTarget,
};
use gpui::{prelude::*, *};
use spaceport_client::{ConnectTarget, protocol::ClientPaneInputEvent};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    task::{Poll, Waker},
};

#[derive(Default)]
pub(crate) enum FileTransfer {
    #[default]
    Idle,
    Uploading(Transfer),
    Cancelling(Transfer),
    Waiting {
        transfer: Transfer,
        text: String,
        wake: WakeOnDrop,
    },
    Accepted(Transfer),
    Cleaning(Transfer),
}

pub(crate) struct Transfer {
    fence: InputFence,
    host: String,
    target: InputTarget,
    label: String,
    cancelled: Arc<AtomicBool>,
    sent: Arc<AtomicU64>,
    total: Arc<AtomicU64>,
    shown: (u64, u64, bool),
}

/// The completion task owns cleanup, even after the waiting window disappears.
pub(crate) struct WakeOnDrop(Waker);

impl Drop for WakeOnDrop {
    fn drop(&mut self) {
        self.0.wake_by_ref();
    }
}

enum Event<'a> {
    Start(Transfer),
    Cancel,
    Poll,
    Uploaded {
        token: &'a Arc<AtomicBool>,
        result: &'a spaceport_client::Result<Vec<String>>,
        waker: &'a Waker,
    },
    Finished(&'a Arc<AtomicBool>),
}

impl FileTransfer {
    pub(crate) fn active(&self) -> Option<&Transfer> {
        match self {
            Self::Idle => None,
            Self::Uploading(transfer)
            | Self::Cancelling(transfer)
            | Self::Waiting { transfer, .. }
            | Self::Accepted(transfer)
            | Self::Cleaning(transfer) => Some(transfer),
        }
    }
}

impl Transfer {
    /// How far the copy is, for the taskbar: busy until its size is known,
    /// and nothing once cancelled.
    #[cfg(any(windows, test))]
    pub(crate) fn progress(&self) -> Option<crate::progress::Progress> {
        use crate::progress::Progress;
        let (sent, total, cancelled) = self.shown;
        if cancelled {
            return None;
        }
        Some(if total == 0 {
            Progress::Busy
        } else {
            Progress::Fraction(transfer_progress(sent, total).0)
        })
    }
}

impl Drop for Transfer {
    fn drop(&mut self) {
        // This worker signal also covers window teardown, outside UI transitions.
        self.cancelled.store(true, Ordering::Release);
    }
}

fn transfer_progress(sent: u64, total: u64) -> (f32, String) {
    let size = |bytes: u64| {
        let mut value = bytes as f64;
        // Selectors of window-input-size.
        let mut unit = "b";
        for next in ["kib", "mib", "gib", "tib", "pib", "eib"] {
            if value < 1024. {
                break;
            }
            value /= 1024.;
            unit = next;
        }
        let number = format!("{value:.1}");
        fl!(
            "window-input-size",
            value = number.trim_end_matches(".0").to_owned(),
            unit = unit
        )
    };
    let fraction = if total == 0 {
        0.
    } else {
        (sent as f64 / total as f64).clamp(0., 1.) as f32
    };
    (
        fraction,
        fl!(
            "window-input-copy-progress",
            sent = size(sent),
            total = size(total),
            percent = format!("{:.0}", fraction * 100.)
        ),
    )
}

#[cfg(any(windows, test))]
impl SpaceportWindow {
    /// The file copy this window's taskbar button reports.
    pub(crate) fn taskbar_progress(&self) -> Option<crate::progress::Progress> {
        self.file_transfer.active().and_then(Transfer::progress)
    }
}

impl SpaceportWindow {
    fn file_transfer_current(&self, transfer: &Transfer) -> bool {
        let endpoint = &self.endpoints[self.selected_endpoint];
        self.fence_holds(&transfer.fence)
            && matches!(&endpoint.connection.target, ConnectTarget::Ssh { target, .. } if target == &transfer.host)
            && self.live.status.is_connected()
            && self
                .live
                .snapshot
                .as_ref()
                .is_some_and(|snapshot| match &transfer.target {
                    InputTarget::Pane(id) => snapshot.panes.iter().any(|pane| &pane.pane_id == id),
                    InputTarget::Popup(_) => true,
                })
            && self
                .live
                .surface
                .as_ref()
                .is_none_or(|surface| match &transfer.target {
                    InputTarget::Pane(id) => {
                        surface.popup.is_none()
                            && surface.panes.iter().any(|pane| &pane.pane_id == id)
                    }
                    InputTarget::Popup(id) => surface
                        .popup
                        .as_ref()
                        .is_some_and(|popup| &popup.terminal_id == id),
                })
    }

    pub(crate) fn poll_file_transfer(&mut self, cx: &mut Context<Self>) {
        self.transition_file_transfer(Event::Poll, cx);
    }

    pub(crate) fn cancel_file_transfer(&mut self, cx: &mut Context<Self>) {
        self.transition_file_transfer(Event::Cancel, cx);
    }

    /// The only writer of transfer phases and their displayed progress.
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 136 lines")]
    fn transition_file_transfer(
        &mut self,
        event: Event<'_>,
        cx: &mut Context<Self>,
    ) -> Option<bool> {
        if let Event::Start(transfer) = event {
            self.file_transfer = FileTransfer::Uploading(transfer);
            cx.notify();
            return None;
        }
        let mut state = std::mem::take(&mut self.file_transfer);
        let transfer = match &mut state {
            FileTransfer::Idle => return Some(false),
            FileTransfer::Uploading(transfer)
            | FileTransfer::Cancelling(transfer)
            | FileTransfer::Waiting { transfer, .. }
            | FileTransfer::Accepted(transfer)
            | FileTransfer::Cleaning(transfer) => transfer,
        };
        if let Event::Uploaded { token, .. } | Event::Finished(token) = &event
            && !Arc::ptr_eq(&transfer.cancelled, token)
        {
            self.file_transfer = state;
            return Some(false);
        }
        if matches!(event, Event::Finished(_)) {
            cx.notify();
            return Some(false);
        }
        let current = self.file_transfer_current(transfer);
        if matches!(event, Event::Cancel) || !current {
            transfer.cancelled.store(true, Ordering::Release);
        }
        let progress = (
            transfer.sent.load(Ordering::Relaxed),
            transfer.total.load(Ordering::Relaxed),
            transfer.cancelled.load(Ordering::Acquire),
        );
        if transfer.shown != progress {
            transfer.shown = progress;
            cx.notify();
        }
        let cancelled = progress.2;
        if cancelled && let FileTransfer::Uploading(transfer) = state {
            state = FileTransfer::Cancelling(transfer);
        }
        let mut error = None;
        let mut cleanup_failed = false;
        if let Event::Uploaded { result, waker, .. } = event {
            match &mut state {
                FileTransfer::Uploading(_) | FileTransfer::Cancelling(_) => {
                    match result {
                        Ok(paths) => match super::file_drop::quote_paths(
                            &paths.iter().map(PathBuf::from).collect::<Vec<_>>(),
                        ) {
                            Ok(text) => {
                                if let FileTransfer::Uploading(transfer)
                                | FileTransfer::Cancelling(transfer) = state
                                {
                                    state = FileTransfer::Waiting {
                                        transfer,
                                        text,
                                        wake: WakeOnDrop(waker.clone()),
                                    };
                                }
                            }
                            Err(source) => error = Some(source.localize()),
                        },
                        Err(source) => {
                            // i18n: data
                            crate::storage_warning::warn_storage_failure("File transfer", source);
                            cleanup_failed =
                                matches!(source, spaceport_client::Error::UploadCleanup { .. });
                            error = Some(source.localize());
                        }
                    }
                }
                FileTransfer::Waiting { wake, .. } => wake.0.clone_from(waker),
                _ => {}
            }
        }
        let ready = self.menu.page.is_none() && self.input_ready();
        let accepted = match &state {
            FileTransfer::Waiting { transfer, text, .. } if cancelled || ready => {
                if cancelled {
                    error = Some(spaceport_client::Error::UploadCancelled.localize());
                    Some(false)
                } else {
                    let result = self.endpoints[self.selected_endpoint]
                        .connection
                        .handle
                        .as_ref()
                        .ok_or(crate::Error::NotConnected)
                        .and_then(|handle| {
                            ConnectionBridge::send_input(
                                handle,
                                &transfer.fence.boot,
                                &transfer.target,
                                ClientPaneInputEvent::Paste(text.clone()),
                            )
                            .map_err(crate::Error::from)
                        });
                    error = result.as_ref().err().map(Localize::localize);
                    Some(result.is_ok())
                }
            }
            FileTransfer::Accepted(_) => Some(true),
            FileTransfer::Cleaning(_) => Some(false),
            _ if error.is_some() => Some(false),
            _ => None,
        };
        if let Some(accepted) = accepted
            && !matches!(state, FileTransfer::Accepted(_) | FileTransfer::Cleaning(_))
        {
            let transfer = match state {
                FileTransfer::Uploading(transfer)
                | FileTransfer::Cancelling(transfer)
                | FileTransfer::Waiting { transfer, .. } => transfer,
                _ => unreachable!(),
            };
            state = if accepted {
                FileTransfer::Accepted(transfer)
            } else {
                FileTransfer::Cleaning(transfer)
            };
            if current && !cleanup_failed {
                self.local_transfer_notice(
                    &if accepted {
                        fl!("window-input-copy-complete")
                    } else if cancelled {
                        fl!("window-input-copy-cancelled")
                    } else {
                        fl!("window-input-copy-failed")
                    },
                    error.unwrap_or_else(|| fl!("window-input-copy-complete-body")),
                    cx,
                );
            }
            cx.notify();
        }
        self.file_transfer = state;
        accepted
    }

    pub(super) fn start_file_transfer(
        &mut self,
        target: InputTarget,
        paths: Vec<PathBuf>,
        cx: &mut Context<Self>,
    ) -> Delivery {
        self.start_file_transfer_with(
            target,
            paths,
            |host, paths, cancelled, progress| {
                spaceport_client::upload_files(host, paths, cancelled, progress)
            },
            spaceport_client::remove_uploaded_files,
            cx,
        )
    }

    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 103 lines")]
    fn start_file_transfer_with(
        &mut self,
        target: InputTarget,
        paths: Vec<PathBuf>,
        upload: impl FnOnce(
            &str,
            &[PathBuf],
            &AtomicBool,
            &mut dyn FnMut(u64, u64),
        ) -> spaceport_client::Result<Vec<String>>
        + Send
        + 'static,
        cleanup: impl FnOnce(&str, &[String]) -> spaceport_client::Result<()> + Send + 'static,
        cx: &mut Context<Self>,
    ) -> Delivery {
        if !self.accepts_remote_images() || paths.is_empty() {
            return Delivery::Refused;
        }
        if self.file_transfer.active().is_some() {
            self.notices.show(
                crate::notice::Cause::Transfer,
                fl!("window-input-files-busy"),
            );
            self.local_transfer_notice(
                &fl!("window-input-copy-not-started"),
                fl!("window-input-copy-not-started-body"),
                cx,
            );
            return Delivery::Refused;
        }
        let endpoint = &self.endpoints[self.selected_endpoint];
        let ConnectTarget::Ssh { target: host, .. } = &endpoint.connection.target else {
            return Delivery::Refused;
        };
        let Some(fence) = self.input_fence() else {
            return Delivery::Refused;
        };
        let host = host.clone();
        let endpoint_label = crate::notifications::safe_text(&endpoint.label, 160);
        let label = if paths.len() == 1 {
            paths[0]
                .file_name()
                .map(|name| crate::notifications::safe_text(&name.to_string_lossy(), 160))
                .unwrap_or_else(|| fl!("window-input-copy-file"))
        } else {
            fl!("window-input-copy-files", count = paths.len())
        };
        let cancelled = Arc::new(AtomicBool::new(false));
        let sent = Arc::new(AtomicU64::new(0));
        let total = Arc::new(AtomicU64::new(0));
        let worker_cancel = cancelled.clone();
        let worker_sent = sent.clone();
        let worker_total = total.clone();
        let worker_host = host.clone();
        let executor = cx.background_executor().clone();
        let work = executor.spawn(async move {
            upload(&worker_host, &paths, &worker_cancel, &mut |sent, total| {
                worker_total.store(total, Ordering::Relaxed);
                worker_sent.store(sent, Ordering::Relaxed);
            })
        });
        let cleanup_host = host.clone();
        let task_cancel = cancelled.clone();
        // The weak entity may disappear before SSH returns. Keep awaiting the
        // result so successful but unclaimed files are still cleaned up.
        cx.spawn(async move |this, cx| {
            let result = work.await;
            let accepted = std::future::poll_fn(|task| {
                match this.update(cx, |this, cx| {
                    this.transition_file_transfer(
                        Event::Uploaded {
                            token: &task_cancel,
                            result: &result,
                            waker: task.waker(),
                        },
                        cx,
                    )
                }) {
                    Ok(None) => Poll::Pending,
                    Ok(Some(accepted)) => Poll::Ready(accepted),
                    Err(_) => Poll::Ready(false),
                }
            })
            .await;
            let cleanup_failed = match result {
                Ok(paths) if !accepted => executor
                    .spawn(async move { cleanup(&cleanup_host, &paths) })
                    .await
                    .is_err(),
                Err(spaceport_client::Error::UploadCleanup { .. }) => true,
                _ => false,
            };
            if cleanup_failed {
                let _ = this.update(cx, |this, cx| {
                    this.local_transfer_notice(
                        &fl!("window-input-cleanup-failed"),
                        fl!("window-input-cleanup-failed-body", device = endpoint_label),
                        cx,
                    );
                });
            }
            let _ = this.update(cx, |this, cx| {
                this.transition_file_transfer(Event::Finished(&task_cancel), cx);
            });
        })
        .detach();
        self.transition_file_transfer(
            Event::Start(Transfer {
                fence,
                host,
                target,
                label,
                cancelled,
                sent,
                total,
                shown: (0, 0, false),
            }),
            cx,
        );
        Delivery::Copying
    }

    pub(super) fn render_file_transfer(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Option<AnyElement> {
        let transfer = self.file_transfer.active()?;
        if self.menu.page.is_some() {
            return None;
        }
        let (sent, total, cancelled) = transfer.shown;
        let (fraction, progress) = transfer_progress(sent, total);
        let title = if cancelled {
            fl!("window-input-copy-cancelling")
        } else if total > 0 && sent >= total {
            fl!("window-input-copy-finishing")
        } else {
            fl!("window-input-copying")
        };
        let accent = self.theme.primary();
        let realm = window.viewport_size().width;
        Some(
            div()
                .id("file-transfer")
                .debug_selector(|| "file-transfer".into())
                .absolute()
                .right(px(12.))
                .top(px(72.))
                .w((realm - px(24.)).max(px(0.)).min(px(340.)))
                .occlude()
                .rounded(px(crate::config::corners::PANEL))
                .border_1()
                .border_color(rgb(accent))
                .bg(rgb(self.theme.surface))
                .text_color(rgb(self.theme.foreground))
                .text_font(&self.config.ui)
                .text_size(px(self.config.ui.size))
                .p(px(12.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
                .on_scroll(|_, _, cx| cx.stop_propagation())
                .child(div().child(title))
                .child(div().truncate().child(transfer.label.clone()))
                .child(
                    div()
                        .debug_selector(|| "file-transfer-track".into())
                        .w_full()
                        .h(px(6.))
                        .rounded(px(crate::config::corners::CONTROL))
                        .overflow_hidden()
                        .bg(rgb(self.theme.active))
                        .child(
                            div()
                                .debug_selector(|| "file-transfer-progress".into())
                                .h_full()
                                .w(relative(fraction))
                                .bg(rgb(accent)),
                        ),
                )
                .child(div().text_color(rgb(self.theme.muted)).child(progress))
                .child(
                    div()
                        .id("cancel-file-transfer")
                        .debug_selector(|| "cancel-file-transfer".into())
                        .cursor_pointer()
                        .child(if cancelled {
                            fl!("window-input-copy-cancel-pressed")
                        } else {
                            fl!("window-input-copy-cancel")
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            cx.stop_propagation();
                            this.cancel_file_transfer(cx);
                        })),
                )
                .into_any_element(),
        )
    }
}

#[cfg(test)]
pub(crate) mod tests;
