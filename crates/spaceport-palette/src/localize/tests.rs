//! Every variant of the palette's own failure words itself for a window.

use crate::Error;
use spaceport_i18n::assertions::assert_localized;
use std::io;

#[test]
fn every_palette_variant_has_a_message() {
    assert_localized(&[
        Error::CommandChanged,
        Error::ConnectionNotReady,
        Error::DestinationRemoved,
        Error::HostUnavailable,
        Error::InvalidFrecency,
        Error::LocalChanged,
        Error::NoSession,
        Error::PaneRemoved,
        Error::ProjectCreate {
            source: io::Error::other("disk on fire"),
        },
        Error::ProjectExists,
        Error::ProjectLimit,
        Error::ProjectPath,
        Error::ProjectRemoved,
        Error::ProjectResponse,
        Error::ProjectRootMissing,
        Error::ProjectStateChanged,
        Error::ProjectVariable("HOME".into()),
        Error::SessionChanged,
        Error::TabRemoved,
        Error::UnsupportedCommand,
        Error::WorkspaceRemoved,
    ]);
}
