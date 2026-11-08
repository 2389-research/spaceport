//! A retired phone greeting remains decodable without a phone client type.
//! GUI and local command-line greetings keep their exact JSON names.

use super::*;
use serde_json::json;

#[test]
fn retired_phone_kind_decodes_as_other() {
    let hello: ClientMessage = serde_json::from_value(json!({
        "type": "hello",
        "client": "remote",
        "revision": REVISION,
        "version": VERSION,
    }))
    .unwrap();
    assert!(matches!(
        hello,
        ClientMessage::Hello {
            client: ClientKind::Other,
            ..
        }
    ));
}

#[test]
fn retained_client_kind_names_stay_on_wire() {
    for (kind, expected) in [(ClientKind::Gui, "gui"), (ClientKind::Cli, "cli")] {
        assert_eq!(serde_json::to_value(kind).unwrap(), json!(expected));
    }
}
