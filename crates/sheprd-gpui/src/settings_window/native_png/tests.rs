// ABOUTME: Checks the PNG budget with actual full native Settings captures.
// ABOUTME: Verifies lossless decoding of every pixel and both original dimensions.
#![allow(clippy::unwrap_used)]
use super::*;

#[test]
fn full_native_settings_captures_fit_without_losing_pixels() {
    for (name, original) in [
        (
            "wide",
            &include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../docs/evidence/review-squad/settings-copy/copy-on-select-dark-960.png"
            ))[..],
        ),
        (
            "narrow",
            &include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../docs/evidence/review-squad/settings-copy/notifications-light-680.png"
            ))[..],
        ),
    ] {
        let native = image::load_from_memory(original).unwrap().to_rgba8();
        let expected = native.clone();
        let png = encode(native).unwrap();
        let decoded = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!(decoded.dimensions(), expected.dimensions());
        assert!(
            decoded.as_raw() == expected.as_raw(),
            "{name} PNG altered native pixels"
        );
        eprintln!(
            "NATIVE_PNG {name} source_bytes={} encoded_bytes={} pixels_equal=true",
            original.len(),
            png.len()
        );
    }
}
