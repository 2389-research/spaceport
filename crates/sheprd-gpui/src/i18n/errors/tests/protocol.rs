use super::*;
use sheprd_client::protocol::{self, Error};
use std::io;

#[test]
fn every_protocol_variant_has_a_message() {
    let errors = every_protocol_error();
    assert_localized(&errors);
    // The codec's own diagnostic is bincode's Debug, kept as it is, so the two
    // variants that carry it sit out the vocabulary check.
    let ours: Vec<Error> = errors
        .into_iter()
        .filter(|error| !matches!(error, Error::Decode(_) | Error::Encode(_)))
        .collect();
    assert_no_wire_words(&ours);
}

/// Every variant of the protocol's error, for the checks above.
fn every_protocol_error() -> Vec<Error> {
    // bincode's errors come from the codec itself: a byte that is no bool,
    // and a frame over its one-byte limit.
    let decode = protocol::decode_payload::<bool>(&[5]).unwrap_err();
    assert!(matches!(decode, Error::Decode(_)), "{decode:?}");
    let encode = protocol::encode_message(&vec![0_u8; 64], 1).unwrap_err();
    assert!(matches!(encode, Error::Encode(_)), "{encode:?}");
    vec![
        Error::CellCount,
        Error::ClipboardImageExtension,
        Error::ClipboardImageSize,
        Error::ClipboardImageTarget,
        Error::CursorBounds,
        decode,
        Error::DeltaGridLimit,
        Error::DeltaMetadataCells,
        Error::DeltaPopup,
        Error::DeltaSpan,
        Error::EmptyFrame,
        encode,
        Error::EncodingJson(serde_json::from_str::<u8>("x").unwrap_err()),
        Error::EncodingLimit,
        Error::EncodingPayload(base64::DecodeError::InvalidLength(3)),
        Error::FrameLimit,
        Error::HyperlinkIndex,
        Error::Io(io::Error::other("disk on fire")),
        Error::PatchCursorBounds,
        Error::PatchGeometry,
        Error::PatchIdentity,
        Error::PatchRowBounds,
        Error::ScrollBounds,
        Error::ScrollCount,
        Error::ScrollPayload,
        Error::ScrollTruncated,
        Error::SurfaceBaseline,
        Error::TrailingBytes,
    ]
}
