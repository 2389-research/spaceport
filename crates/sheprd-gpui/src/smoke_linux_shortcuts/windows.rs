// ABOUTME: Owns the exact additional window used for native Cycle Windows qualification.
// ABOUTME: Verifies private attachment and defers preparation and normal closure outside root borrows.
use super::{Baseline, Command, SheprdWindow, baseline, coherent, same_members, same_runtime};
use anyhow::{Context as _, Result, ensure};
use gpui::{AnyWindowHandle, App, Context, Window, WindowHandle};
use sheprd_client::ConnectTarget;
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Copy)]
enum Creation {
    // keep-sorted start
    Failed,
    Opened(WindowHandle<SheprdWindow>),
    Pending,
    // keep-sorted end
}

pub(super) struct Opening {
    original: AnyWindowHandle,
    result: Rc<Cell<Creation>>,
    target: ConnectTarget,
}

pub(super) struct Owned {
    original: AnyWindowHandle,
    other: WindowHandle<SheprdWindow>,
    target: ConnectTarget,
}

pub(super) fn open(
    view: &SheprdWindow,
    window: &Window,
    cx: &mut Context<SheprdWindow>,
) -> Result<Opening> {
    ensure!(
        cx.windows() == [window.window_handle()],
        "native window preparation requires only its original window"
    );
    let target = view
        .endpoints
        .first()
        .context("native window has no endpoint")?
        .connection
        .target
        .clone();
    ensure!(
        matches!(target, ConnectTarget::Socket(_)),
        "native additional window requires the explicit private socket"
    );
    let result = Rc::new(Cell::new(Creation::Pending));
    let opened = Rc::clone(&result);
    let failed = Rc::clone(&result);
    crate::app::open_additional_window_then(
        target.clone(),
        None,
        move |_, window, _| {
            opened.set(
                window
                    .window_handle()
                    .downcast::<SheprdWindow>()
                    .map_or(Creation::Failed, Creation::Opened),
            )
        },
        move |_| failed.set(Creation::Failed),
        cx,
    );
    Ok(Opening {
        original: window.window_handle(),
        result,
        target,
    })
}

impl Opening {
    pub(super) fn poll(&self, before: &Baseline, cx: &mut App) -> Result<Option<Owned>> {
        let other = match self.result.get() {
            Creation::Pending => return Ok(None),
            Creation::Failed => anyhow::bail!("native additional window creation failed"),
            Creation::Opened(handle) => handle,
        };
        let owned = Owned {
            original: self.original,
            other,
            target: self.target.clone(),
        };
        if owned.inspect(before, cx)?.is_none() {
            return Ok(None);
        }
        owned.check_addition(before, &cx.windows())?;
        Ok(Some(owned))
    }
}

impl Owned {
    pub(super) fn check_addition(
        &self,
        before: &Baseline,
        windows: &[AnyWindowHandle],
    ) -> Result<()> {
        ensure!(
            windows.len() == before.windows.len() + 1
                && before.windows.iter().all(|handle| windows.contains(handle))
                && windows.contains(&AnyWindowHandle::from(self.other)),
            "native additional window changed unexpected window identities"
        );
        Ok(())
    }
    fn inspect(&self, before: &Baseline, cx: &mut App) -> Result<Option<bool>> {
        self.other
            .update(cx, |view, window, cx| -> Result<_> {
                ensure!(
                    view.endpoints
                        .first()
                        .is_some_and(|endpoint| endpoint.connection.target == self.target),
                    "native second window attached to another endpoint"
                );
                ensure!(
                    view.notices.is_none() && view.live.notices.is_none(),
                    "native second window has a client notice"
                );
                if !coherent(view) || view.config_load.is_some() {
                    return Ok(None);
                }
                let after = baseline(view, window, cx)?;
                same_runtime(before, &after)?;
                same_members(before, &after)?;
                view.live
                    .surface
                    .as_ref()
                    .context("native second window has no surface")?
                    .frame
                    .validate()?;
                Ok(Some(window.is_window_active()))
            })
            .context("observing the exact native second window")?
    }

    pub(super) fn activate_original(&self, cx: &mut App) {
        let original = self.original;
        cx.defer(move |cx| {
            if let Err(error) = original.update(cx, |_, window, _| window.activate_window()) {
                eprintln!("LINUX_WINDOW_ACTIVATE_ERROR {error:#}");
            }
        });
    }

    pub(super) fn inactive(&self, cx: &mut App) -> Result<bool> {
        self.other
            .update(cx, |_, window, _| !window.is_window_active())
            .context("checking native second window preparation")
    }

    pub(super) fn observed(&self, before: &Baseline, cx: &mut App) -> Result<bool> {
        let active = self.inspect(before, cx)? == Some(true);
        if active {
            eprintln!(
                "LINUX_KEYS_CASE cycle_target_active=true explicit_socket=true coherent_target=true runtime_members_preserved=true"
            );
        }
        Ok(active)
    }

    pub(super) fn request_close(&self, cx: &mut App) {
        let other = self.other;
        cx.defer(move |cx| {
            let result = other
                .update(cx, |view, window, cx| -> Result<()> {
                    ensure!(
                        view.menu.page.is_none(),
                        "native owned window has a popup before cleanup"
                    );
                    view.command(Command::CloseWindow, window, cx);
                    Ok(())
                })
                .context("closing the exact native second window")
                .and_then(|result| result);
            if let Err(error) = result {
                eprintln!("LINUX_WINDOW_CLOSE_ERROR {error:#}");
            }
        });
    }

    pub(super) fn closed(&self, cx: &App) -> bool {
        !cx.windows().contains(&AnyWindowHandle::from(self.other))
    }

    pub(super) fn check_removal(&self, before: &Baseline, after: &Baseline) -> Result<()> {
        let expected = before
            .windows
            .iter()
            .copied()
            .filter(|handle| *handle != AnyWindowHandle::from(self.other))
            .collect::<Vec<_>>();
        ensure!(
            after.windows == expected && after.windows.contains(&self.original),
            "native cleanup changed windows beyond its owned second window"
        );
        Ok(())
    }
}
