//! Machine views come from Core's catalog, including its connection state.
//! An absent method is reported as unsupported.
use super::{MockCore, connected, every_method, hello_reply, pair};
use crate::{Core, Error, Method};
use serde_json::json;
use std::thread;

#[test]
fn machine_list_reads_core_owned_catalog() {
    let (client, mut core) = connected();
    let call = thread::spawn(move || client.machine_list());
    let request = core.request();
    assert_eq!(request["method"], "machine.list");
    assert_eq!(request["params"], json!({}));
    core.answer(
        &request,
        json!({"machines": [{
            "name": "lab", "target": "lab.example", "id": "core-2",
            "host": "lab-host", "version": "0.1.18", "state": "needs_login",
            "since_ms": 42, "why": "authentication required", "temporary": true,
            "future_field": "kept by Core"
        }]}),
    );
    let machines = call.join().unwrap().unwrap().machines;
    assert_eq!(machines.len(), 1);
    assert_eq!(machines[0].machine.name, "lab");
    assert_eq!(machines[0].state, "needs_login");
    assert_eq!(machines[0].since_ms, Some(42));
    assert_eq!(machines[0].why.as_deref(), Some("authentication required"));
    assert!(machines[0].machine.temporary);
}

#[test]
fn absent_machine_list_is_unsupported() {
    let (client, server) = pair();
    let methods: Vec<&str> = every_method()
        .into_iter()
        .filter(|name| *name != "machine.list")
        .collect();
    let core = thread::spawn(move || MockCore::accept(server, &hello_reply("0.1.18", &methods)));
    let client = Core::over(client).unwrap();
    let (_core, _) = core.join().unwrap();
    assert!(matches!(
        client.machine_list(),
        Err(Error::Unsupported(Method::MachineList))
    ));
}

#[test]
fn terminal_get_preserves_the_qualified_core_target() {
    let (client, mut core) = connected();
    let call = thread::spawn(move || client.terminal_get("lab/t7"));
    let request = core.request();
    assert_eq!(request["method"], "terminal.get");
    assert_eq!(request["params"], json!({"terminal": "lab/t7"}));
    core.answer(&request, super::terminal("lab/t7"));
    assert_eq!(call.join().unwrap().unwrap().address, "lab/t7");
}

#[test]
fn machine_add_sends_only_a_core_ssh_target_without_install() {
    let (client, mut core) = connected();
    let call =
        thread::spawn(move || client.machine_add("user@lab.invalid", Some("lab"), None, false));
    let request = core.request();
    assert_eq!(request["method"], "machine.add");
    assert_eq!(
        request["params"],
        json!({"target": "user@lab.invalid", "name": "lab", "temporary": false})
    );
    core.answer(
        &request,
        json!({
            "name": "lab", "target": "user@lab.invalid", "id": "core-2",
            "host": "lab-host", "version": "0.1.18", "temporary": false,
            "via": false, "session": null
        }),
    );
    let machine = call.join().unwrap().unwrap();
    assert_eq!(machine.name, "lab");
    assert_eq!(machine.id, "core-2");
}
