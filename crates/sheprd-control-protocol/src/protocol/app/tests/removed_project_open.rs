use super::*;

#[test]
fn removed_folder_open_request_is_not_decoded() {
    for invalid in [
        r#"{"method":"app","caller":{},"op":{"op":"open","path":"/a"}}"#,
        r#"{"method":"app","caller":{},"op":{"op":"open","path":"/a","cwd":"/b"}}"#,
    ] {
        assert!(
            serde_json::from_str::<Request>(invalid).is_err(),
            "{invalid}"
        );
    }
    let focus = r#"{"method":"app","caller":{},"op":{"op":"focus"}}"#;
    assert!(serde_json::from_str::<Request>(focus).is_ok());
}
