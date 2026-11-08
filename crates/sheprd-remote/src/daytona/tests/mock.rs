//! A scripted Daytona API on loopback, for exercising the real HTTP path.
#![allow(clippy::unwrap_used)]

use crate::cloud::tests::server::{Request, serve};
use crate::daytona::{
    Error, Settings,
    api::{Readiness, wait_ready},
    http,
};
use secrecy::{ExposeSecret, SecretString};
use std::sync::{Arc, Mutex};

fn account(
    handler: impl Fn(&Request) -> (u16, String) + Send + 'static,
) -> (Settings, Arc<Mutex<Vec<Request>>>) {
    let (base, log) = serve(handler);
    let mut settings = super::settings();
    settings.base = format!("{base}/api");
    settings.organization = Some("org-1".into());
    settings.snapshot = Some("daytonaio/sandbox:0.4.3".into());
    (settings, log)
}

fn sandbox(state: &str) -> String {
    format!(r#"{{"id":"s1","name":"herdr-calm-otter","state":"{state}","target":"us","extra":1}}"#)
}

#[test]
fn listing_sends_the_key_and_organization() {
    let (settings, log) =
        account(
            |request| match (request.method.as_str(), request.target.as_str()) {
                ("GET", "/api/sandbox?limit=100") => (
                    200,
                    format!(r#"{{"items":[{}],"nextCursor":null}}"#, sandbox("started")),
                ),
                other => panic!("unexpected {other:?}"),
            },
        );
    let key = SecretString::from("dtn_fixture");
    let request = http::Request {
        settings: &settings,
        key: &key,
    };
    assert_eq!(request.sandboxes().unwrap().len(), 1);
    let log = log.lock().unwrap();
    let list = &log[0];
    assert_eq!(list.authorization.as_deref(), Some("Bearer dtn_fixture"));
    assert!(list.body.is_empty());
    assert!(
        list.headers
            .to_ascii_lowercase()
            .contains("x-daytona-organization-id: org-1")
    );
}

#[test]
fn waiting_starts_a_stopped_sandbox_once_and_issues_a_short_lived_token() {
    let polls = Arc::new(Mutex::new(0));
    let counter = polls.clone();
    let (settings, log) = account(move |request| {
        match (request.method.as_str(), request.target.as_str()) {
        ("GET", "/api/sandbox/s1") => {
            let mut polls = counter.lock().unwrap();
            *polls += 1;
            (200, sandbox(match *polls { 1 => "stopped", 2 => "starting", _ => "started" }))
        }
        ("POST", "/api/sandbox/s1/start") => (200, sandbox("starting")),
        ("POST", "/api/sandbox/s1/ssh-access?expiresInMinutes=10") => (
            200,
            r#"{"id":"a1","sandboxId":"s1","token":"tok","sshCommand":"ssh tok@ssh.app.daytona.io","expiresAt":"x","createdAt":"x","updatedAt":"x"}"#.into(),
        ),
        other => panic!("unexpected {other:?}"),
    }
    });
    let key = SecretString::from("dtn_fixture");
    let request = http::Request {
        settings: &settings,
        key: &key,
    };
    let mut seen = Vec::new();
    let ready = wait_ready(
        &request,
        "s1",
        || false,
        |readiness| seen.push(readiness.clone()),
    )
    .unwrap();
    assert_eq!(ready.name, "herdr-calm-otter");
    assert_eq!(
        seen,
        [Readiness::Stopped, Readiness::Pending("starting".into())]
    );
    let access = request.ssh_access("s1").unwrap();
    assert_eq!(access.token.expose_secret(), "tok");
    let log = log.lock().unwrap();
    assert_eq!(
        log.iter().filter(|r| r.target.ends_with("/start")).count(),
        1,
        "start exactly once"
    );
}

#[test]
fn a_missing_sandbox_reads_as_deleted_and_a_rejected_key_as_authentication() {
    let (settings, _) = account(|request| match request.target.as_str() {
        "/api/sandbox/gone" => (404, r#"{"message":"Sandbox not found"}"#.into()),
        _ => (401, r#"{"message":"Unauthorized"}"#.into()),
    });
    let key = SecretString::from("dtn_fixture");
    let request = http::Request {
        settings: &settings,
        key: &key,
    };
    assert!(matches!(
        wait_ready(&request, "gone", || false, |_| {}),
        Err(Error::Deleted)
    ));
    assert!(matches!(request.sandboxes(), Err(Error::Authentication)));
    assert!(matches!(
        wait_ready(&request, "s1", || true, |_| {}),
        Err(Error::Cancelled)
    ));
}
