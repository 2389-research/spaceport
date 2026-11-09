use super::*;

#[test]
fn disconnected_receivers_cancel_without_waiting_for_the_deadline() {
    let (_address, server, client) = pair();
    let (sender, events) = mpsc::sync_channel::<Event>(QUEUE);
    let (finished, done) = mpsc::channel();
    let worker = thread::spawn(move || {
        serve(&server, &sender);
        finished.send(()).unwrap();
    });
    write_line(&client, &waiting_events(600)).unwrap();
    let Event::Request(incoming) = events.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(incoming.is_live());
    drop(client);
    done.recv_timeout(Duration::from_secs(5)).unwrap();
    worker.join().unwrap();
    assert!(!incoming.is_live());
    let response = Response::Done {
        message: "late".into(),
    };
    assert_eq!(incoming.try_respond(response.clone()), Err(response));
}

#[test]
fn failed_socket_writes_do_not_enqueue_another_request() {
    let (_address, server, client) = pair();
    let (sender, events) = mpsc::sync_channel::<Event>(QUEUE);
    drop(client);
    finish_response(
        &server,
        Response::Done {
            message: "unread".into(),
        },
    );
    drop(sender);
    assert!(events.try_recv().is_err());
}
