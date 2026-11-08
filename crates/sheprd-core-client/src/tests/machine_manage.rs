//! Catalog edits target Core's saved machine name and preserve its identity.
//! A rename or removal is one Core request; callers do not retry it blindly.
use super::{MockCore, connected, every_method, hello_reply, pair};
use crate::{Core, Error, Method};
use serde_json::json;
use std::thread;

#[test]
fn machine_rename_uses_core_names_and_returns_the_same_identity() {
    let (client, mut core) = connected();
    let call = thread::spawn(move || client.machine_rename("lab", "work"));
    let request = core.request();
    assert_eq!(request["method"], "machine.rename");
    assert_eq!(
        request["params"],
        json!({"machine": "lab", "new_name": "work"})
    );
    core.answer(
        &request,
        json!({
            "name": "work", "target": "fixture-host", "id": "core-2",
            "host": "lab-host", "version": "0.1.18"
        }),
    );
    let machine = call.join().unwrap().unwrap();
    assert_eq!(machine.name, "work");
    assert_eq!(machine.id, "core-2");
}

#[test]
fn machine_remove_is_one_exact_core_request() {
    let (client, mut core) = connected();
    let call = thread::spawn(move || client.machine_rm("work"));
    let request = core.request();
    assert_eq!(request["method"], "machine.rm");
    assert_eq!(request["params"], json!({"machine": "work"}));
    core.answer(&request, json!({"removed": "work"}));
    assert_eq!(call.join().unwrap().unwrap(), "work");
}

#[test]
fn absent_machine_rename_is_unsupported_before_dispatch() {
    let (client, server) = pair();
    let methods: Vec<&str> = every_method()
        .into_iter()
        .filter(|name| *name != "machine.rename")
        .collect();
    let core = thread::spawn(move || MockCore::accept(server, &hello_reply("0.1.18", &methods)));
    let client = Core::over(client).unwrap();
    let (_core, _) = core.join().unwrap();
    assert!(matches!(
        client.machine_rename("lab", "work"),
        Err(Error::Unsupported(Method::MachineRename))
    ));
}
