//! Document calls preserve Core's exact machine target and CAS version.
//! A remote document stays in that Core, separate from the local document.
use super::{MockCore, connected, every_method, hello_reply, pair};
use crate::{Core, Error, Method, hello::Mismatch};
use serde_json::json;
use std::thread;

#[test]
fn document_get_targets_one_saved_machine() {
    let (client, mut core) = connected();
    let call = thread::spawn(move || client.doc_get("spaceport", Some("lab")));
    let request = core.request();
    assert_eq!(request["method"], "doc.get");
    assert_eq!(request["params"], json!({"name": "spaceport", "on": "lab"}));
    core.answer(
        &request,
        json!({"value": {"schema_version": 1}, "version": 7}),
    );
    let doc = call.join().unwrap().unwrap();
    assert_eq!(doc.version, 7);
    assert_eq!(doc.value, json!({"schema_version": 1}));
}

#[test]
fn document_set_keeps_machine_and_expected_version() {
    let (client, mut core) = connected();
    let call = thread::spawn(move || {
        client.doc_set("spaceport", &json!({"schema_version": 1}), 7, Some("lab"))
    });
    let request = core.request();
    assert_eq!(request["method"], "doc.set");
    assert_eq!(
        request["params"],
        json!({
            "name": "spaceport", "on": "lab", "value": {"schema_version": 1}, "version": 7
        })
    );
    core.answer(&request, json!({"version": 8}));
    assert_eq!(call.join().unwrap().unwrap(), 8);
}

#[test]
fn absent_document_write_method_refuses_the_connection() {
    let (client, server) = pair();
    let methods: Vec<&str> = every_method()
        .into_iter()
        .filter(|name| *name != "doc.set")
        .collect();
    let core = thread::spawn(move || MockCore::accept(server, &hello_reply("0.1.18", &methods)));
    let result = Core::over(client);
    let (_core, _) = core.join().unwrap();
    assert!(matches!(
        result,
        Err(Error::Compat(Mismatch::Missing(methods))) if methods == vec![Method::DocSet]
    ));
}
