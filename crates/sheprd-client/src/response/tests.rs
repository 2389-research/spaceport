use super::*;
use serde_json::json;

fn read(response: &Value) -> String {
    match tagged(response, "ok") {
        Ok(result) => format!("ok {result}"),
        Err(Error::Endpoint { code, message }) => format!("endpoint {code:?} {message:?}"),
        Err(Error::ResponseSchema(_)) => "schema".into(),
        Err(Error::ResponseMissingResult) => "missing".into(),
        Err(Error::ResponseType) => "type".into(),
        Err(error) => format!("other {error}"),
    }
}

#[test]
fn each_envelope_shape_has_one_reading() {
    let failed = |error: Value| json!({"id": "r", "error": error});
    for (response, reading) in [
        (
            json!({"id": "r", "result": {"type": "ok"}}),
            r#"ok {"type":"ok"}"#,
        ),
        // A null error is no error.
        (
            json!({"id": "r", "error": null, "result": {"type": "ok"}}),
            r#"ok {"type":"ok"}"#,
        ),
        (json!({"id": "r", "error": null}), "missing"),
        (json!({"id": "r", "error": null, "result": null}), "missing"),
        (json!({"id": "r"}), "missing"),
        (json!("not an envelope"), "schema"),
        (json!({"id": "r", "result": {"type": "other"}}), "type"),
        (json!({"id": "r", "result": "untagged"}), "type"),
        // An error wins over a result sent beside it.
        (
            json!({"error": {"code": "denied", "message": "no"}, "result": {"type": "ok"}}),
            r#"endpoint Other("denied") "no""#,
        ),
        (
            failed(json!({"code": "stale_content", "message": "changed"})),
            r#"endpoint StaleContent "changed""#,
        ),
        // Lenient bodies keep whatever text the daemon sent.
        (
            failed(json!({"code": "denied"})),
            r#"endpoint Other("denied") "denied""#,
        ),
        (
            failed(json!({"message": "no"})),
            r#"endpoint Other("endpoint_error") "no""#,
        ),
        (
            failed(json!("busy")),
            r#"endpoint Other("endpoint_error") "busy""#,
        ),
        (
            failed(json!({"code": 7, "message": ["no"]})),
            r#"endpoint Other("endpoint_error") """#,
        ),
        (failed(json!(42)), r#"endpoint Other("endpoint_error") """#),
        (
            failed(json!(false)),
            r#"endpoint Other("endpoint_error") """#,
        ),
        // Control and direction-override characters never reach the display.
        (
            failed(json!({"code": "de\nnied", "message": "bad\u{1b}[31m\n\u{202e}news"})),
            r#"endpoint Other("denied") "bad[31mnews""#,
        ),
    ] {
        assert_eq!(read(&response), reading, "{response}");
    }
}

#[test]
fn messages_and_codes_are_bounded() {
    let error = result(&json!({"error": {
        "code": "c".repeat(MAX_CODE_CHARS + 1),
        "message": "é".repeat(MAX_MESSAGE_CHARS + 1),
    }}))
    .unwrap_err();
    let Error::Endpoint { code, message } = &error else {
        panic!("{error:?}");
    };
    assert_eq!(code.as_str().len(), MAX_CODE_CHARS);
    assert_eq!(message.chars().count(), MAX_MESSAGE_CHARS);
}

#[test]
fn a_refusal_without_text_still_displays() {
    let silent = result(&json!({"error": {}})).unwrap_err();
    assert_eq!(silent.to_string(), "Invalid daemon error");
    let spoken = result(&json!({"error": "busy"})).unwrap_err();
    assert_eq!(spoken.to_string(), "busy");
}

#[test]
fn known_codes_round_trip_and_unknown_ones_stay_open() {
    for code in [
        EndpointErrorCode::PaneNotFound,
        EndpointErrorCode::PluginLinkFailed,
        EndpointErrorCode::QueryTooLarge,
        EndpointErrorCode::StaleContent,
        EndpointErrorCode::Other("future_code".into()),
    ] {
        assert_eq!(EndpointErrorCode::from(code.as_str().to_owned()), code);
    }
}

#[test]
fn typed_decodes_a_tagged_result_and_keeps_the_schema_source() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Made {
        id: String,
    }
    let made = json!({"result": {"type": "made", "id": "w1", "later": true}});
    assert_eq!(
        typed::<Made>(&made, "made").unwrap(),
        Made { id: "w1".into() }
    );
    assert!(matches!(
        typed::<Made>(&made, "other"),
        Err(Error::ResponseType)
    ));
    let error = typed::<Made>(&json!({"result": {"type": "made"}}), "made").unwrap_err();
    assert!(matches!(error, Error::ResponseSchema(_)));
    assert!(std::error::Error::source(&error).is_some());
}
