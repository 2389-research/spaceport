use super::*;
use sheprd_remote::Error;
use std::io;

mod teleport;

fn io() -> io::Error {
    io::Error::other("disk on fire")
}

fn json() -> serde_json::Error {
    serde_json::from_str::<u8>("x").unwrap_err()
}

fn client() -> sheprd_client::Error {
    sheprd_client::Error::ScriptTimeout
}

#[test]
fn wrapped_errors_delegate() {
    assert_localized(&[Error::Client(client())]);
}

#[test]
fn every_variant_has_a_message() {
    let errors = vec![
        Error::PreferencesPath,
        Error::Teleport(sheprd_remote::teleport::Error::Cancelled),
    ];
    assert_localized(&errors);
}

#[test]
fn system_text_is_shown_as_it_is() {
    assert_verbatim(&[Error::Io(io()), Error::Json(json())]);
    assert_localized(&[Error::Path(
        sheprd_client::local_path::PathError::MissingHome,
    )]);
}
