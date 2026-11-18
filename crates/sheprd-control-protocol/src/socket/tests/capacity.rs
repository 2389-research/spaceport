// ABOUTME: Exercises bounded wait admission through real private control connections.
// ABOUTME: Keeps quick commands available when the waiting connection allowance is full.
use super::*;

#[test]
fn eight_parked_waits_leave_capacity_for_quick_commands() {
    let address = address();
    let server = Server::bind(&address.path).unwrap();
    let mut waiting = Vec::new();
    let mut parked = Vec::new();
    for _ in 0..8 {
        let path = address.path.clone();
        waiting.push(thread::spawn(move || call(&path, &waiting_events(30))));
        parked.push(server.next().expect("each waiting request reaches the UI"));
    }
    assert!(matches!(
        call(&address.path, &waiting_events(30)).unwrap(),
        Response::Error {
            code: ErrorCode::Busy,
            ..
        }
    ));
    let next = thread::spawn({
        let path = address.path.clone();
        move || call(&path, &focus_request())
    });
    let incoming = server.next();
    for incoming in parked {
        incoming.respond(Response::done("finished waiting"));
    }
    for waiting in waiting {
        assert!(waiting.join().unwrap().is_ok());
    }
    assert!(
        incoming.is_some(),
        "parked waits consumed quick request capacity"
    );
    incoming.unwrap().respond(Response::done("focused"));
    assert_eq!(next.join().unwrap().unwrap(), Response::done("focused"));
}
