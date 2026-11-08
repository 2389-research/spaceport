use super::*;

#[gpui::test]
fn status_refuses_a_missing_explicit_window_over_the_local_socket(cx: &mut gpui::TestAppContext) {
    let address = socket::test_address();
    let server = socket::Server::bind(&address.path).unwrap();
    let path = address.path.clone();
    let caller = std::thread::spawn(move || {
        socket::call(
            &path,
            &Request::Status(Caller {
                window: Some(42),
                ..Caller::default()
            }),
        )
    });
    let incoming = server.next().unwrap();
    cx.update(|cx| serve(incoming, &mut State::default(), cx));
    assert!(matches!(
        caller.join().unwrap().unwrap(),
        Response::Error {
            code: ErrorCode::NotFound,
            ..
        }
    ));
}
