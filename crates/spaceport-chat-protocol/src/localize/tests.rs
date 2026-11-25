//! Every variant of this crate's failures words itself for a window.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::{Cursor, EntryId, Error, PermissionId, Rejection};
use spaceport_i18n::assertions::assert_localized;

#[test]
fn every_chat_protocol_variant_has_a_message() {
    assert_localized(&[
        Error::ActivityLimit,
        Error::ActivityText,
        Error::DuplicatePermission(PermissionId(4)),
        Error::EmptyFrame,
        Error::EntryOrder {
            id: EntryId(2),
            last: EntryId(5),
        },
        Error::FrameLimit { len: 9_000_000 },
        Error::InvalidEndpoint,
        Error::Io(std::io::Error::other("disk on fire")),
        Error::Json(serde_json::from_str::<u8>("x").unwrap_err()),
        Error::NoModes,
        Error::NotText(EntryId(3)),
        Error::QuestionAnswersLimit(EntryId(7)),
        Error::StreamGap {
            after: None,
            got: Cursor {
                epoch: uuid::Uuid::nil(),
                seq: 2,
            },
        },
        Error::UnknownEntry(EntryId(8)),
        Error::UnknownPermission(PermissionId(9)),
    ]);
}

#[test]
fn every_chat_rejection_has_a_message() {
    assert_localized(&[
        Rejection::EmptyPrompt,
        Rejection::PromptTooLong,
        Rejection::NotReady,
        Rejection::NotWorking,
        Rejection::UnknownPermission,
        Rejection::UnknownOption,
        Rejection::UnknownMode,
        Rejection::UnknownSetting,
        Rejection::UnknownQueued,
        Rejection::AgentUnavailable,
        Rejection::TooManyAttachments,
        Rejection::QueueFull,
        Rejection::UnknownFile,
        Rejection::Unsupported,
        Rejection::Other,
    ]);
}
