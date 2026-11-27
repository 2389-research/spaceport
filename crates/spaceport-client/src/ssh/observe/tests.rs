// ABOUTME: Checks that observation only forwards to a compatible, running named session.
// ABOUTME: Covers the default target and paths that cannot cross the line-based handshake.

use super::*;
use serde_json::json;

#[test]
fn accepts_only_the_requested_running_session() {
    let client = json!({"endpoint_protocol_generation":1, "endpoint_capabilities":["surface_interest","presentation_effects_fence","health_check"], "remote_host_bridge":true});
    let mut server = json!({"running":true,"endpoint_compatible":true,"session":"dev","socket":"/tmp/a b/herdr.sock"});
    let banner = |s: &serde_json::Value| format!("{client}\nherdr-observer:{s}\n");
    let socket =
        |s: &serde_json::Value, session: &str| client_socket(banner(s).as_bytes(), session);
    assert_eq!(
        socket(&server, "dev").as_deref(),
        Some("/tmp/a b/herdr-client.sock")
    );
    assert_eq!(socket(&server, "other"), None);
    server["session"] = serde_json::Value::Null;
    assert!(socket(&server, "default").is_some());
    server["running"] = json!(false);
    assert_eq!(socket(&server, "default"), None);
    server["running"] = json!(true);
    server["socket"] = json!("/tmp/injected\npath/herdr.sock");
    assert_eq!(socket(&server, "default"), None);
}
