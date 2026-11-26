//! Every variant of the sound worker's own failure words itself.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use spaceport_i18n::assertions::{assert_localized, assert_verbatim};
use std::io;

#[test]
fn every_sound_variant_has_a_message() {
    assert_localized(&[
        Error::Cancelled,
        Error::ConfigSize,
        Error::Decode(rodio::decoder::DecoderError::UnrecognizedFormat),
        Error::Device(rodio::DeviceSinkError::NoDevice),
        Error::File {
            path: "/tmp/ding.mp3".into(),
            source: io::Error::other("disk on fire"),
        },
        Error::FileSize,
        Error::Path {
            path: "/tmp/view.toml".into(),
            source: Box::new(Error::ConfigSize),
        },
        Error::Stream(rodio::cpal::StreamError::DeviceNotAvailable),
        Error::Timeout,
    ]);
    // Settings text is the system's or the parser's, shown as it is.
    assert_verbatim(&[
        Error::Io(io::Error::other("disk on fire")),
        Error::Toml(toml::from_str::<toml::Table>("=").unwrap_err()),
    ]);
}
