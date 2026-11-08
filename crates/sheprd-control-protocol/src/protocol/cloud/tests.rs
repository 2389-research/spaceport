#![allow(clippy::unwrap_used)]
use super::*;

#[test]
fn cloud_shape_keeps_its_provider_wire_name() {
    assert_eq!(
        serde_json::to_string(&Shape::Sandbox).unwrap(),
        "\"sandbox\""
    );
    assert_eq!(
        serde_json::from_str::<Shape>("\"machine\"").unwrap(),
        Shape::Machine
    );
}
