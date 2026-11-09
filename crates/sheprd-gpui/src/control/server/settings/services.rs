//! Integrations reported by the selected host through its existing connection.
use super::{super::window_for, Watches, socket};
use crate::{
    SheprdWindow,
    control::protocol::{
        Caller, ErrorCode, Response,
        settings::{IntegrationInfo, Reply},
    },
};
use gpui::App;
use std::time::Duration;

fn no_window() -> Response {
    Response::error(ErrorCode::NoWindow, "Spaceport has no window open")
}

pub(super) fn integrations(
    caller: &Caller,
    wait: Duration,
    incoming: socket::Incoming,
    watches: &mut Watches,
    cx: &mut App,
) {
    let handle = match window_for(caller, cx) {
        Ok(handle) => handle,
        Err(response) => {
            incoming.respond(response);
            return;
        }
    };
    if handle
        .update(cx, |view, _, cx| view.load_integrations(cx))
        .is_err()
    {
        incoming.respond(no_window());
        return;
    }
    watches.add(
        incoming,
        wait,
        move |cx| match handle.read_with(cx, |view, _| listed(view)) {
            Ok(answer) => answer,
            Err(_) => Some(Response::error(ErrorCode::NoWindow, "The window closed")),
        },
        Response::error(
            ErrorCode::Timeout,
            "The host did not list its integrations in time",
        ),
        cx,
    );
}

fn listed(view: &SheprdWindow) -> Option<Response> {
    let integrations = &view.integrations;
    if integrations.busy {
        return None;
    }
    if let Some(error) = &integrations.error {
        return Some(Response::error(ErrorCode::Failed, error.clone()));
    }
    Some(Response::Settings {
        reply: Reply::Integrations {
            integrations: integrations
                .listed()
                .map(|(id, label, status)| IntegrationInfo {
                    id: id.to_owned(),
                    label: label.to_owned(),
                    status: status.to_owned(),
                })
                .collect(),
        },
    })
}
