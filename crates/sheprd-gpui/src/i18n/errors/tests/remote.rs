use super::*;
use sheprd_remote::Error;
use std::io;

#[cfg(feature = "cloud")]
mod cloud;
#[cfg(feature = "container")]
mod container;
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

/// A boxed `herdr-remote` error, as providers keep their saved device's.
#[cfg(feature = "cloud")]
fn remote() -> Box<Error> {
    Box::new(Error::NoConfig)
}

#[test]
fn wrapped_errors_delegate() {
    assert_localized(&[Error::Client(client())]);
}

#[test]
fn every_variant_has_a_message() {
    let errors = vec![
        Error::Config(Box::new(crate::Error::ConnectionBusy)),
        Error::NoConfig,
        Error::PreferencesPath,
        Error::Teleport(sheprd_remote::teleport::Error::Cancelled),
        #[cfg(feature = "cloud")]
        Error::Cloud(sheprd_remote::cloud::Error::Missing),
        #[cfg(feature = "cloud")]
        Error::SandboxImage(sheprd_remote::cloud::image::Error::Busy),
    ];
    assert_localized(&errors);
}

#[test]
fn system_text_is_shown_as_it_is() {
    assert_verbatim(&[
        Error::Config("an unknown loader failure".into()),
        Error::Io(io()),
        Error::Json(json()),
    ]);
    assert_localized(&[Error::Path(
        sheprd_client::local_path::PathError::MissingHome,
    )]);
}
