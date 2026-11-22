//! Core text reads keep the target machine and path on the wire.
//! They do not stand in for binary download or file replacement.
use super::{MockCore, connected, every_method, hello_reply, pair};
use crate::{Core, Error, Method, wire::FileRead};
use serde_json::json;
use std::thread;

#[test]
fn text_read_preserves_exact_machine_id_path_and_line_bounds() {
    let (client, mut core) = connected();
    let call = thread::spawn(move || {
        client.file_read(&FileRead {
            path: "/fixture/report.txt",
            on: Some("core-id-9"),
            cwd: None,
            offset: Some(2),
            limit: Some(1),
        })
    });
    let request = core.request();
    assert_eq!(request["method"], "file.read");
    assert_eq!(
        request["params"],
        json!({
            "path": "/fixture/report.txt", "on": "core-id-9", "offset": 2, "limit": 1
        })
    );
    core.answer(
        &request,
        json!({"text": "second\n", "first": 2, "last": 2, "lines": 3}),
    );
    let read = call.join().unwrap().unwrap();
    assert_eq!(
        (read.text.as_str(), read.first, read.last, read.lines),
        ("second\n", 2, 2, 3)
    );
}

#[test]
fn absent_file_read_route_is_unsupported_before_dispatch() {
    let (client, server) = pair();
    let methods: Vec<&str> = every_method()
        .into_iter()
        .filter(|name| *name != "file.read")
        .collect();
    let core = thread::spawn(move || MockCore::accept(server, &hello_reply("0.1.18", &methods)));
    let client = Core::over(client).unwrap();
    let (_core, _) = core.join().unwrap();
    assert!(matches!(
        client.file_read(&FileRead {
            path: "/fixture/report.txt",
            on: None,
            cwd: None,
            offset: None,
            limit: None,
        }),
        Err(Error::Unsupported(Method::FileRead))
    ));
}
