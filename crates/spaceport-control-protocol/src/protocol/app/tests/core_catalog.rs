use super::*;
use crate::protocol::Response;

#[test]
fn status_core_catalog_keeps_unknown_distinct_from_an_empty_result() {
    let current = Response::Status(Status {
        version: "0.1".into(),
        protocol: crate::protocol::PROTOCOL_VERSION,
        pid: 1,
        windows: vec![],
        core_catalog: CoreCatalog {
            epoch: Some(7),
            state: CoreCatalogState::Current,
            machine_count: Some(0),
            error: None,
        },
    });
    let value = serde_json::to_value(&current).unwrap();
    assert!(value.get("browser_tabs").is_none());
    assert_eq!(value["core_catalog"]["state"], "current");
    assert_eq!(value["core_catalog"]["machine_count"], 0);
    assert_eq!(serde_json::from_value::<Response>(value).unwrap(), current);

    let old = r#"{"status":"status","version":"0.1","protocol":11,"pid":1,"browser_tabs":false,"windows":[]}"#;
    let Response::Status(old) = serde_json::from_str(old).unwrap() else {
        panic!("expected status");
    };
    assert_eq!(old.core_catalog.state, CoreCatalogState::NotStarted);
    assert_eq!(old.core_catalog.machine_count, None);
}
