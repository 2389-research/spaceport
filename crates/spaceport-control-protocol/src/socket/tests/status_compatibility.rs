// ABOUTME: Checks version refusal before decoding a newer status payload.
// ABOUTME: Uses real framed socket responses with unknown future status fields.
use super::*;

#[test]
fn a_newer_status_schema_reports_its_protocol_before_decoding_fields() {
    let address = address();
    let listener = transport::Listener::bind(&address.path).unwrap();
    let version = crate::protocol::PROTOCOL_VERSION + 1;
    let peer = thread::spawn(move || {
        let mut stream = listener.accept().unwrap();
        stream.prepare(IO_TIMEOUT).unwrap();
        let request: Request = serde_json::from_slice(&read_line(&stream).unwrap()).unwrap();
        assert!(matches!(request, Request::Status(_)));
        write_line(
            &stream,
            &serde_json::json!({
                "status": "status", "protocol": version,
                "version": "future", "pid": 1, "windows": [],
                "core_catalog": { "state": "future_state" }
            }),
        )
        .unwrap();
    });
    let result = call(&address.path, &Request::Status(Caller::default()));
    peer.join().unwrap();
    assert!(
        matches!(result, Err(crate::Error::Rejected {
            code: ErrorCode::Unsupported, ref message
        }) if message.contains(&format!("control protocol {version}"))
            && message.contains("update the older")),
        "version refusal must precede status schema decoding: {result:?}"
    );
}
