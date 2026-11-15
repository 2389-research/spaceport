//! Following a parsed link in a window: switching to a device or showing a
//! notification's pane.
// i18n: migrated

use super::Link;
use crate::{
    SheprdWindow,
    control::{ErrorCode, Response},
    i18n::{FluentValue, fl},
};
use gpui::{App, Context, Global, Window, WindowHandle};

/// A link the app was started with, followed once a window can.
#[derive(Default)]
pub(crate) struct Pending(Option<Link>);

impl Global for Pending {}

/// Why a link was not followed. The command line reports it in English,
/// as all its output is; the window shows it in its own language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Refusal {
    Closed,
    DialogOpen,
    NoDevice(String),
    NotificationGone,
}

impl Refusal {
    fn code(&self) -> ErrorCode {
        match self {
            Self::Closed => ErrorCode::NoWindow,
            Self::DialogOpen => ErrorCode::WindowBusy,
            Self::NoDevice(_) | Self::NotificationGone => ErrorCode::NotFound,
        }
    }

    /// The message saying why, with its arguments.
    fn message(&self) -> (&'static str, Vec<(&'static str, FluentValue<'static>)>) {
        match self {
            Self::Closed => ("link-refused-closed", Vec::new()),
            Self::DialogOpen => ("link-refused-dialog-open", Vec::new()),
            Self::NoDevice(id) => (
                "link-refused-no-device",
                vec![("device", FluentValue::from(id.clone()))],
            ),
            Self::NotificationGone => ("link-refused-notification-gone", Vec::new()),
        }
    }

    pub(crate) fn response(&self) -> Response {
        let (id, args) = self.message();
        Response::error(self.code(), crate::i18n::english(id, &args))
    }

    /// The reason in the window's language.
    pub(crate) fn shown(&self) -> String {
        let (id, args) = self.message();
        crate::i18n::message(id, &args)
    }
}

/// Puts a failed startup link's complaint on the window's status line.
fn complain(view: &mut SheprdWindow, text: String, cx: &mut Context<SheprdWindow>) {
    view.local_error = Some(text);
    cx.notify();
}

pub(crate) fn defer(link: Link, cx: &mut App) {
    cx.set_global(Pending(Some(link)));
}

/// Follows the link the app was started with once this window has what it
/// needs: a device once the saved devices are loaded (a window starts with
/// Local alone). A notification from before the app started is not known, so
/// starting the app was all its click could do.
pub(crate) fn poll(view: &SheprdWindow, window: &Window, cx: &mut Context<SheprdWindow>) {
    let ready = match cx
        .try_global::<Pending>()
        .and_then(|pending| pending.0.as_ref())
    {
        None => return,
        Some(Link::Notification { .. }) => {
            cx.global_mut::<Pending>().0 = None;
            return;
        }
        Some(Link::Device { id }) => {
            view.catalog.loaded() || view.endpoints.iter().any(|endpoint| &endpoint.id == id)
        }
    };
    let Some(handle) = window.window_handle().downcast::<SheprdWindow>() else {
        return;
    };
    if !ready {
        return;
    }
    let Some(link) = cx.global_mut::<Pending>().0.take() else {
        return;
    };
    cx.defer(move |cx| {
        if let Err(refusal) = follow_link(link, handle, cx) {
            let _ = handle.update(cx, |view, _, cx| {
                complain(view, fl!("link-open-failed", reason = refusal.shown()), cx);
            });
        }
    });
}

/// Follows `link` for the command line, which reports in English.
pub(crate) fn follow(link: Link, handle: WindowHandle<SheprdWindow>, cx: &mut App) -> Response {
    follow_link(link, handle, cx).unwrap_or_else(|refusal| refusal.response())
}

/// Follows `link`, answering what was done in the command line's English,
/// or why it was not.
fn follow_link(
    link: Link,
    handle: WindowHandle<SheprdWindow>,
    cx: &mut App,
) -> Result<Response, Refusal> {
    match link {
        Link::Notification { tag } => {
            if crate::window::system_notifications::open(&tag, cx) {
                // i18n: data
                Ok(Response::done("Opened the notification's pane"))
            } else {
                Err(Refusal::NotificationGone)
            }
        }
        Link::Device { id } => handle
            .update(cx, |view, window, cx| {
                if view.menu.page.is_some() {
                    return Err(Refusal::DialogOpen);
                }
                if view.select_endpoint(&id, cx).is_err() {
                    return Err(Refusal::NoDevice(id));
                }
                bring_forward(window, cx);
                Ok(Response::done(format!(
                    // i18n: data
                    "Showing {}",
                    view.endpoints[view.selected_endpoint].label
                )))
            })
            .unwrap_or(Err(Refusal::Closed)),
    }
}

fn bring_forward(window: &mut Window, cx: &mut App) {
    cx.activate(true);
    window.activate_window();
}

#[cfg(test)]
#[path = "follow_tests.rs"]
mod tests;
