// ABOUTME: Checks size errors from real outbound message encoding.
// ABOUTME: Keeps shared diagnostics accurate for both local and received messages.
#![allow(clippy::unwrap_used)]
use sheprd_protocol::{Error, decode_payload, encode_message};

#[test]
fn an_empty_local_message_does_not_blame_the_daemon() {
    let error = encode_message(&(), 16).unwrap_err();
    assert!(matches!(error, Error::EmptyFrame));
    assert_eq!(error.to_string(), "A message cannot be empty.");
}

#[test]
fn an_empty_received_payload_keeps_the_shared_size_error_accurate() {
    let error = decode_payload::<()>(&[]).unwrap_err();
    assert!(matches!(error, Error::FrameLimit));
    assert_eq!(
        error.to_string(),
        "The message is outside the allowed size limits."
    );
}

#[test]
fn an_oversized_local_message_does_not_blame_the_daemon() {
    let error = encode_message(&[0_u8; 16], 8).unwrap_err();
    let Error::Encode(bincode::error::EncodeError::Io { inner, .. }) = error else {
        panic!("outbound size error lost its encoder cause: {error:?}");
    };
    let limit = inner.get_ref().unwrap().downcast_ref::<Error>().unwrap();
    assert!(matches!(limit, Error::FrameLimit));
    assert_eq!(
        limit.to_string(),
        "The message is outside the allowed size limits."
    );
}
