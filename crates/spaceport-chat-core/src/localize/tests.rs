//! Every variant of this crate's failures words itself for a window.

use crate::{Error, env, session::Refusal, widgets};
use spaceport_chat_protocol::{ByeReason, Cursor, Rejection};
use spaceport_i18n::assertions::assert_localized;
use std::io;

fn protocol() -> spaceport_chat_protocol::Error {
    spaceport_chat_protocol::Error::EmptyFrame
}

fn cursor(seq: u64) -> Cursor {
    Cursor {
        epoch: uuid::Uuid::nil(),
        seq,
    }
}

#[test]
fn every_chat_variant_has_a_message() {
    let mut errors = vec![
        Error::Apply(protocol()),
        Error::Busy,
        Error::Closed,
        Error::Connect(io::Error::other("refused")),
        Error::Environment(env::Error::Name {
            name: "HERDR=1".into(),
        }),
        Error::Environment(env::Error::Size),
        Error::Handshake,
        Error::NoEndpointDir,
        Error::NotAttached,
        Error::Overflow,
        Error::Read(protocol()),
        Error::ReadOnly,
        Error::Remote(spaceport_client::Error::Disconnected),
        Error::Spawn(io::Error::other("no threads")),
        Error::Stream {
            after: Some(cursor(4)),
            got: cursor(6),
        },
        Error::Stream {
            after: None,
            got: cursor(2),
        },
        Error::Version(7),
    ];
    errors.extend(
        [
            ByeReason::Exiting,
            ByeReason::Handshake,
            ByeReason::Version { supported: 2 },
        ]
        .map(Error::Bye),
    );
    assert_localized(&errors);
}

#[test]
fn every_permission_answer_refusal_has_a_message() {
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

#[test]
fn every_delivery_refusal_has_an_honest_localized_message() {
    assert_localized(&[
        Refusal::DeliveryUnconfirmed,
        Refusal::Remote(Rejection::Unsupported),
        Refusal::SessionChanged,
    ]);
}

#[test]
fn every_interactive_card_variant_has_a_message() {
    assert_localized(&[
        widgets::Error::ActionGone,
        widgets::Error::Component,
        widgets::Error::ComponentCount,
        widgets::Error::FieldId,
        widgets::Error::FormValue,
        widgets::Error::Json(serde_json::from_str::<u8>("x").unwrap_err()),
        widgets::Error::Title,
        widgets::Error::TooLarge,
        widgets::Error::TooManyFields,
    ]);
}
