use super::*;
use crate::chat::{Error, StartError, env, page, pasted, visual, widgets};
use spaceport_chat_protocol::{Agent, ByeReason, Cursor};
use std::{io, sync::Arc};

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
    let mut attach = vec![
        spaceport_chat_core::Error::Apply(protocol()),
        spaceport_chat_core::Error::Busy,
        spaceport_chat_core::Error::Closed,
        spaceport_chat_core::Error::Connect(io::Error::other("refused")),
        spaceport_chat_core::Error::Environment(env::Error::Name {
            name: "HERDR=1".into(),
        }),
        spaceport_chat_core::Error::Environment(env::Error::Size),
        spaceport_chat_core::Error::Handshake,
        spaceport_chat_core::Error::NoEndpointDir,
        spaceport_chat_core::Error::NotAttached,
        spaceport_chat_core::Error::Overflow,
        spaceport_chat_core::Error::Read(protocol()),
        spaceport_chat_core::Error::ReadOnly,
        spaceport_chat_core::Error::Remote(spaceport_client::Error::Disconnected),
        spaceport_chat_core::Error::Spawn(io::Error::other("no threads")),
        spaceport_chat_core::Error::Stream {
            after: Some(cursor(4)),
            got: cursor(6),
        },
        spaceport_chat_core::Error::Stream {
            after: None,
            got: cursor(2),
        },
        spaceport_chat_core::Error::Version(7),
    ];
    attach.extend(
        [
            ByeReason::Exiting,
            ByeReason::Handshake,
            ByeReason::Version { supported: 2 },
        ]
        .map(spaceport_chat_core::Error::Bye),
    );
    let mut errors: Vec<Error> = attach.into_iter().map(Error::Attach).collect();
    errors.extend([
        Error::Environment(env::Error::Size),
        Error::Pasted(pasted::Error::Folder),
        Error::Start(StartError::Disabled),
    ]);
    assert_localized(&errors);
}

#[test]
fn every_chat_start_variant_has_a_message() {
    let mut errors = vec![
        StartError::CoreTerminalRequired,
        StartError::Disabled,
        StartError::NotConnected,
        StartError::NotLocal,
        StartError::NoWorkspace,
        StartError::ProgramMissing,
        StartError::ProgramUnknown,
        StartError::Request(Arc::new(crate::Error::NoTab)),
        StartError::Response,
        StartError::SlotBusy,
        StartError::Timeout,
    ];
    errors.extend(Agent::ALL.iter().copied().map(StartError::AgentMissing));
    assert_localized(&errors);
}

#[test]
fn every_pasted_image_variant_has_a_message() {
    assert_localized(&[
        pasted::Error::Folder,
        pasted::Error::Write(io::Error::other("disk full")),
    ]);
}

#[test]
fn every_chat_page_variant_has_a_message() {
    assert_localized(&[
        page::Error::Folder,
        page::Error::NotFile,
        page::Error::TooLarge,
    ]);
    assert_verbatim(&[page::Error::Io(io::Error::other("permission denied"))]);
}

#[test]
fn every_chat_visual_variant_has_a_message() {
    use base64::Engine;
    let boxed = || -> Box<dyn std::error::Error + Send + Sync> { "bad".into() };
    let base64 = base64::engine::general_purpose::STANDARD
        .decode("!")
        .unwrap_err();
    let parse = ratex_parser::parse(r"\frac{").unwrap_err();
    assert_localized(&[
        visual::Error::Base64(base64),
        visual::Error::Decode(image::ImageError::IoError(io::Error::other("short"))),
        visual::Error::Diagram(boxed()),
        visual::Error::Fetch(Box::new(ureq::Error::ConnectionFailed)),
        visual::Error::MathParse(parse),
        visual::Error::Math("unknown command".into()),
        visual::Error::MathLimit,
        visual::Error::NotAnImage,
        visual::Error::Panicked,
        visual::Error::Raster(boxed()),
    ]);
}

#[test]
fn every_chat_rejection_has_a_message() {
    use spaceport_chat_protocol::Rejection;
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
