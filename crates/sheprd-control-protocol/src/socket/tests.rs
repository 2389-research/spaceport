#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::protocol::{
    Call, Caller, MAX_WAIT_SECONDS,
    events::{EventsOp, Reply},
};
use test_support::{Address, address};

mod cancellation;
mod framing;
mod identity;
#[cfg(windows)]
mod pipe;
#[cfg(unix)]
mod unix;

fn reload() -> Request {
    Request::Reload(Caller::default())
}

fn waiting_events(wait_seconds: u64) -> Request {
    Request::Events(Call {
        caller: Caller::default(),
        op: EventsOp::Poll {
            after: None,
            instance: None,
            wait_seconds,
            kinds: Vec::new(),
        },
    })
}

/// Both ends of one connection: the server's, timeouts set, and a caller's.
fn pair() -> (Address, Connection, Connection) {
    let address = address();
    let listener = transport::Listener::bind(&address.path).unwrap();
    let client = transport::connect(&address.path, IO_TIMEOUT, IO_TIMEOUT).unwrap();
    let mut server = listener.accept().unwrap();
    server.prepare(IO_TIMEOUT).unwrap();
    (address, server, client)
}

#[test]
fn a_request_reaches_the_ui_and_its_answer_reaches_the_caller() {
    let address = address();
    let server = Server::bind(&address.path).unwrap();
    let caller = thread::spawn({
        let path = address.path.clone();
        move || call(&path, &reload())
    });
    let incoming = server.next().unwrap();
    assert_eq!(incoming.request, reload());
    let expected = Response::Reloaded { tabs: 1 };
    incoming.respond(expected.clone());
    assert_eq!(caller.join().unwrap().unwrap(), expected);
}

#[test]
fn malformed_and_oversized_requests_are_answered_without_the_ui() {
    let address = address();
    let server = Server::bind(&address.path).unwrap();
    // Exactly one byte over, so the server reads all of it and closing
    // never discards unread input, which some kernels answer with a reset.
    for request in [b"not json\n".to_vec(), vec![b'x'; MAX_MESSAGE + 1]] {
        let stream = transport::connect(&address.path, IO_TIMEOUT, IO_TIMEOUT).unwrap();
        (&stream).write_all(&request).unwrap();
        let response: Response = serde_json::from_slice(&read_line(&stream).unwrap()).unwrap();
        assert!(
            matches!(
                response,
                Response::Error {
                    code: ErrorCode::InvalidRequest,
                    ..
                }
            ),
            "{response:?}"
        );
    }
    assert!(server.drain().next().is_none());
}

#[test]
fn a_live_server_keeps_its_address() {
    let address = address();
    let _server = Server::bind(&address.path).unwrap();
    assert!(matches!(
        Server::bind(&address.path),
        Err(crate::Error::SocketInUse { .. })
    ));
}

#[test]
fn a_waiting_caller_does_not_hold_up_the_next_one() {
    let address = address();
    let server = Server::bind(&address.path).unwrap();
    let waiting = thread::spawn({
        let path = address.path.clone();
        move || call(&path, &waiting_events(30))
    });
    let parked = server.next().unwrap();
    assert!(matches!(parked.request, Request::Events(_)));
    // While the first caller waits for the user, another is answered.
    let next = thread::spawn({
        let path = address.path.clone();
        move || call(&path, &reload())
    });
    server
        .next()
        .unwrap()
        .respond(Response::Reloaded { tabs: 0 });
    assert_eq!(
        next.join().unwrap().unwrap(),
        Response::Reloaded { tabs: 0 }
    );
    let expired = Response::Events {
        reply: Reply::Events {
            events: Vec::new(),
            next: 0,
            instance: 0,
        },
    };
    parked.respond(expired.clone());
    assert_eq!(waiting.join().unwrap().unwrap(), expired);
}

#[test]
fn event_waits_are_bounded() {
    let wait = |wait_seconds| answer_timeout(&waiting_events(wait_seconds));
    assert_eq!(wait(5), ANSWER_TIMEOUT + Duration::from_secs(5));
    assert_eq!(
        wait(u64::MAX),
        ANSWER_TIMEOUT + Duration::from_secs(MAX_WAIT_SECONDS)
    );
    assert_eq!(answer_timeout(&reload()), ANSWER_TIMEOUT);
}

#[test]
fn an_answer_over_the_bound_is_refused() {
    let address = address();
    let server = Server::bind(&address.path).unwrap();
    let caller = thread::spawn({
        let path = address.path.clone();
        move || call(&path, &reload())
    });
    server.next().unwrap().respond(Response::Done {
        message: "n".repeat(MAX_MESSAGE),
    });
    assert!(matches!(
        caller.join().unwrap().unwrap(),
        Response::Error {
            code: ErrorCode::Failed,
            ..
        }
    ));
    assert!(server.drain().next().is_none());
}

#[test]
fn a_missing_server_is_reported_as_unavailable() {
    let address = address();
    assert!(matches!(
        call(&address.path, &reload()),
        Err(crate::Error::Unavailable { .. })
    ));
}
