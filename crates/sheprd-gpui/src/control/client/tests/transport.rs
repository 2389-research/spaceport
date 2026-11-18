// ABOUTME: Checks transport failures against the public CLI exit contract.
// ABOUTME: Keeps disconnection, timeout and incompatible replies distinct from refusals.
use super::*;

#[test]
fn closed_connections_report_that_the_app_is_gone() {
    let error = crate::Error::from(crate::control::Error::NoResponse);
    assert_eq!(status(&error), exit::NOT_RUNNING);
    assert_eq!(code(&error), "not_running");
}

#[test]
fn an_unexpected_response_reports_an_incompatible_app() {
    let error = unexpected(Response::done("a reply from another operation"));
    assert_eq!(status(&error), exit::UNSUPPORTED);
    assert_eq!(code(&error), "unsupported");
}

#[test]
fn transport_timeouts_keep_their_exit_code_and_source() {
    use std::{error::Error as _, io};
    for kind in [io::ErrorKind::TimedOut, io::ErrorKind::WouldBlock] {
        let error = crate::Error::from(crate::control::Error::Io(kind.into()));
        assert_eq!(status(&error), exit::TIMEOUT);
        assert_eq!(code(&error), "timeout");
        assert!(error.source().is_some());
    }
}

#[test]
fn broken_transports_report_a_disconnection() {
    use std::io::ErrorKind;
    for kind in [
        ErrorKind::BrokenPipe,
        ErrorKind::ConnectionReset,
        ErrorKind::ConnectionAborted,
        ErrorKind::UnexpectedEof,
    ] {
        let error = crate::Error::from(crate::control::Error::Io(kind.into()));
        assert_eq!(status(&error), exit::NOT_RUNNING);
    }
}

#[test]
fn unknown_response_shapes_report_an_incompatible_app() {
    let source = serde_json::from_str::<Response>(r#"{"status":"future_response"}"#).unwrap_err();
    let error = crate::Error::from(crate::control::Error::Json(source));
    assert_eq!(status(&error), exit::UNSUPPORTED);
}

#[test]
fn a_truncated_response_reports_a_disconnection() {
    let source =
        serde_json::from_str::<Response>(r#"{"status":"done","message":"cut"#).unwrap_err();
    let error = crate::Error::from(crate::control::Error::Json(source));
    assert_eq!(status(&error), exit::NOT_RUNNING);
}
