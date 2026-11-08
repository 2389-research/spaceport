//! Terminal attachments stage bytes on Core's exact target machine.
//! The path returned by Core is data; staging does not type into the terminal.
use super::{MockCore, connected, every_method, hello_reply, pair};
use crate::{Core, Error, Method};
use serde_json::json;
use std::thread;

#[test]
fn put_preserves_the_qualified_terminal_and_binary_bytes() {
    let (client, mut core) = connected();
    let call = thread::spawn(move || client.terminal_put("lab/t7", "diagram.png", b"\0\xffA"));
    let request = core.request();
    assert_eq!(request["method"], "terminal.put");
    assert_eq!(
        request["params"],
        json!({
            "terminal": "lab/t7", "name": "diagram.png", "data": "AP9B"
        })
    );
    core.answer(
        &request,
        json!({"path": "/private/files/diagram.png", "bytes": 3}),
    );
    let put = call.join().unwrap().unwrap();
    assert_eq!(put.path, "/private/files/diagram.png");
    assert_eq!(put.bytes, 3);
}

#[test]
fn oversized_put_is_refused_before_dispatch() {
    let (client, _core) = connected();
    let bytes = vec![0; crate::MAX_PUT_BYTES + 1];
    assert!(matches!(
        client.terminal_put("lab/t7", "oversized.bin", &bytes),
        Err(Error::PutTooLarge { size, limit }) if size == bytes.len() && limit == crate::MAX_PUT_BYTES
    ));
}

#[test]
fn absent_put_route_is_unsupported_before_dispatch() {
    let (client, server) = pair();
    let methods: Vec<&str> = every_method()
        .into_iter()
        .filter(|name| *name != "terminal.put")
        .collect();
    let core = thread::spawn(move || MockCore::accept(server, &hello_reply("0.1.18", &methods)));
    let client = Core::over(client).unwrap();
    let (_core, _) = core.join().unwrap();
    assert!(matches!(
        client.terminal_put("lab/t7", "note.txt", b"hi"),
        Err(Error::Unsupported(Method::TerminalPut))
    ));
}
