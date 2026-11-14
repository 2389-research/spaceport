// ABOUTME: Preserves runtime and connection coverage distinctions in CLI text and JSON.
// ABOUTME: Prevents unavailable support or capacity from appearing as invented hosts.
use super::*;

#[test]
fn coverage_output_names_connections_and_runtime_support_separately() {
    let result = super::super::super::observation::output(Reply::Sessions {
        sessions: vec![],
        coverage: vec![
            Coverage::Runtime {
                name: "Herdr Core".into(),
                unavailable: UnavailableReason::Disabled,
            },
            Coverage::Connection {
                window: 1,
                host: "Local".into(),
                unavailable: Some(UnavailableReason::Disconnected),
            },
            Coverage::Capacity,
        ],
        coverage_truncated: false,
        next_offset: None,
        observed_at_ms: 1,
    })
    .unwrap();
    assert!(result.text.contains("Runtime Herdr Core: Disabled"));
    assert!(result.text.contains("Connection Local: Disconnected"));
    assert!(result.text.contains("Observation capacity reached"));
    assert_eq!(result.json["coverage"][0]["scope"], "runtime");
    assert!(result.json["coverage"][0].get("host").is_none());
    assert_eq!(result.json["coverage"][1]["scope"], "connection");
}
