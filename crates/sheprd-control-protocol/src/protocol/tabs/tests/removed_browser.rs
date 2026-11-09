use super::*;

#[test]
fn page_actions_and_filtered_listing_are_rejected() {
    for wire in [
        r#"{"op":"list","pages":true}"#,
        r#"{"op":"navigate","id":1,"url":"https://a.test/"}"#,
        r#"{"op":"back","id":1}"#,
        r#"{"op":"forward","id":1}"#,
        r#"{"op":"text","id":1}"#,
        r#"{"op":"screenshot","id":1,"path":"/tmp/page.png"}"#,
        r#"{"op":"zoom","id":1,"change":{"change":"in"}}"#,
    ] {
        assert!(serde_json::from_str::<TabOp>(wire).is_err(), "{wire}");
    }
    for wire in [
        r#"{"kind":"text","text":"page"}"#,
        r#"{"kind":"zoom","id":1,"percent":110}"#,
    ] {
        assert!(serde_json::from_str::<Reply>(wire).is_err(), "{wire}");
    }
}
