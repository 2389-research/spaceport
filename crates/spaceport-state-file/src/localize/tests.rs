// ABOUTME: Checks the state file's failures word themselves for a window.
// ABOUTME: Keeps the system's and the parser's own text as it is.

use super::*;
use spaceport_i18n::assertions::{assert_localized, assert_verbatim};
use std::{io, path::PathBuf};

fn path() -> PathBuf {
    PathBuf::from("/home/ada/.local/state/spaceport/browser-tabs.json")
}

#[test]
fn every_state_file_variant_has_a_message() {
    assert_localized(&[
        Error::MissingRoot(path()),
        Error::Size {
            path: path(),
            limit: 4096,
        },
    ]);
    // The system's and the parser's own words, shown as they are.
    assert_verbatim(&[
        Error::Io(io::Error::other("disk on fire")),
        Error::Json(serde_json::from_str::<u8>("x").unwrap_err()),
    ]);
}
