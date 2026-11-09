//! The control socket's messages: one JSON request line, one JSON response
//! line, then the connection closes. Most request families send
//! `{"method", "caller", "op"}` and receive a family-specific reply.
pub mod app;
pub mod chat;
pub mod events;
pub mod hosts;
pub mod settings;
pub mod tabs;
pub mod work;

use serde::{Deserialize, Serialize};

/// Raised whenever a request or reply changes shape, so `status` can tell
/// an outdated app from a refused request. 3: `browser.open` placement.
/// 4: `link`. 5: the `tab` family's `zoom`. 6: the `cloud` family. 7: `chat`.
/// 8: legacy `host` `automation_approve`. 9: `remote`. 10: `settings` `language`.
/// 11: `mod`. 12: `status.core_catalog`. 13: removed phone control.
/// 14: removed local `browser.open`. 15: removed named-session `window.new`.
/// 16: removed `tab.code`. 17: removed `tab.edit`.
/// 18: removed cloud control routes.
/// 19: removed `TabKind::Code`.
/// 20: removed GitHub and PR control routes.
/// 21: removed review open control route.
/// 22: removed orchestrator open control route.
/// 23: removed the native review tab kind from tab listings.
/// 24: removed checkpoint restore from the work control route.
/// 25: removed the Security activity panel and its UI control destination.
/// 26: removed px0 tab open and editor tab kind.
/// 27: removed the work fan-out request.
/// 28: window listings report selected Core presentation group identity.
/// 29: removed native PR, issue, and task activity panel control targets.
/// 30: removed `app.open` and `work.script.archive` requests.
/// 31: removed design feedback and page-note requests and responses.
/// 32: removed worktree script control requests.
/// 33: removed the browser reload request and reply.
/// 34: removed page-only tab operations and replies.
pub const PROTOCOL_VERSION: u32 = 34;

/// Bounds a request and a response alike.
pub const MAX_MESSAGE: usize = 256 * 1024;
/// The longest control wait, in seconds.
pub const MAX_WAIT_SECONDS: u64 = 600;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "method", deny_unknown_fields)]
pub enum Request {
    #[serde(rename = "status")]
    Status(Caller),
    #[serde(rename = "app")]
    App(Call<app::AppOp>),
    #[serde(rename = "window")]
    Window(Call<app::WindowOp>),
    #[serde(rename = "ui")]
    Ui(Call<app::UiOp>),
    #[serde(rename = "command")]
    Command(Call<app::CommandOp>),
    #[serde(rename = "tab")]
    Tab(Call<tabs::TabOp>),
    #[serde(rename = "work")]
    Work(Call<work::WorkOp>),
    #[serde(rename = "host")]
    Host(Call<hosts::HostOp>),
    #[serde(rename = "settings")]
    Settings(Call<settings::SettingsOp>),
    #[serde(rename = "events")]
    Events(Call<events::EventsOp>),
    #[serde(rename = "link")]
    Link(Call<LinkOp>),
    #[serde(rename = "chat")]
    Chat(Call<chat::ChatOp>),
}

/// A `herdr-gpui://` link the shell started a second copy of the app with.
/// The app parses it again: any local process can send one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkOp {
    pub url: String,
}

/// A request of a command family: who asks, and what.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Call<T> {
    pub caller: Caller,
    pub op: T,
}

impl Request {
    /// How long the app may take to answer, beyond the usual bound: some
    /// requests wait for the user, a page, or an event on purpose.
    pub fn wait_seconds(&self) -> u64 {
        let wait = match self {
            Self::Events(call) => call.op.wait_seconds(),
            Self::Tab(call) => call.op.wait_seconds(),
            Self::Host(call) => call.op.wait_seconds(),
            Self::Settings(call) => call.op.wait_seconds(),
            _ => 0,
        };
        wait.min(MAX_WAIT_SECONDS)
    }
}

/// Who is asking: the Herdr pane, workspace, and daemon the caller runs in.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Caller {
    /// The caller's `HERDR_PANE_ID`.
    #[serde(default)]
    pub pane_id: Option<String>,
    /// The Herdr workspace to open the tab in, normally the caller's own
    /// `HERDR_WORKSPACE_ID`. Absent, the tab opens in the workspace the
    /// frontmost window shows.
    #[serde(default)]
    pub workspace_id: Option<String>,
    /// The client socket of the caller's daemon, which tells two local
    /// sessions' identical workspace IDs apart.
    #[serde(default)]
    pub daemon_socket: Option<String>,
    /// The window to act in, by the ID `window list` reports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidRequest,
    InvalidUrl,
    NoWindow,
    WorkspaceNotFound,
    TabLimit,
    Busy,
    Timeout,
    Unsupported,
    /// The window, tab, host, checkpoint, or other thing named is not there.
    NotFound,
    /// The app tried and the operation failed, for example a Git error.
    Failed,
    /// A dialog or menu holds the window, so it takes no commands now.
    WindowBusy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Response {
    Status(app::Status),
    /// Done; the message says what happened, for a person.
    Done {
        message: String,
    },
    App {
        reply: app::Reply,
    },
    Tab {
        reply: tabs::Reply,
    },
    Work {
        reply: work::Reply,
    },
    Host {
        reply: hosts::Reply,
    },
    Settings {
        reply: settings::Reply,
    },
    Events {
        reply: events::Reply,
    },
    Chat {
        reply: chat::Reply,
    },
    Error {
        code: ErrorCode,
        message: String,
    },
}

impl Response {
    pub fn error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::Error {
            code,
            message: message.into(),
        }
    }

    pub fn done(message: impl Into<String>) -> Self {
        Self::Done {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    mod removed_browser;
    mod removed_phone;

    #[test]
    fn removed_browser_open_request_is_rejected() {
        let request = r#"{"method":"browser.open","page":{"url":"https://a.test/"},"caller":{},"focus":true}"#;
        assert!(serde_json::from_str::<Request>(request).is_err());
    }

    #[test]
    fn removed_design_feedback_requests_are_rejected() {
        for request in [
            r#"{"method":"browser.feedback","pane_id":"w_1:p1","wait_seconds":0}"#,
            r#"{"method":"notes.send","caller":{"pane_id":"w_1:p1"},"text":"Picked: B"}"#,
        ] {
            assert!(
                serde_json::from_str::<Request>(request).is_err(),
                "{request}"
            );
        }
    }

    #[test]
    fn requests_have_a_stable_wire_form() {
        let status: Request =
            serde_json::from_str(r#"{"method":"status","pane_id":"w_1:p1"}"#).unwrap();
        assert!(matches!(
            status,
            Request::Status(Caller {
                pane_id: Some(_),
                ..
            })
        ));
    }

    #[test]
    fn unknown_methods_and_fields_are_rejected() {
        for invalid in [
            r#"{"method":"browser.eval","page":{"url":"https://a.test"},"caller":{},"focus":true}"#,
            r#"{"method":"browser.open","page":{"url":"https://a.test"},"caller":{},"focus":true,"extra":1}"#,
            r#"{"method":"browser.open","page":{"script":"x"},"caller":{},"focus":true}"#,
            r#"{"method":"browser.open","page":{"url":"https://a.test"},"caller":{},"focus":true,"placement":"popup"}"#,
            r#"{"method":"browser.open","page":{"url":"https://a.test"},"caller":{"shell":"x"},"focus":true}"#,
            r#"{"method":"browser.open","focus":true}"#,
            r#"{"method":"browser.feedback","wait_seconds":1}"#,
            r#"{"method":"notes.send","caller":{}}"#,
            r#"{"method":"notes.send","caller":{},"text":"x","paste":true}"#,
            r#"["browser.open"]"#,
        ] {
            assert!(
                serde_json::from_str::<Request>(invalid).is_err(),
                "{invalid}"
            );
        }
    }

    #[test]
    fn responses_round_trip() {
        for response in [
            Response::error(ErrorCode::WorkspaceNotFound, "No window shows w_9"),
            Response::done("ok"),
        ] {
            let text = serde_json::to_string(&response).unwrap();
            assert_eq!(serde_json::from_str::<Response>(&text).unwrap(), response);
        }
        assert_eq!(
            serde_json::to_string(&Response::error(ErrorCode::Busy, "x")).unwrap(),
            r#"{"status":"error","code":"busy","message":"x"}"#
        );
    }
}
