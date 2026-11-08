//! Opt-in: check that a wrong boxd key is refused by the API and exec relay.
//! This test uses only an explicitly configured live account.

use super::super::{
    http::Request,
    relay::{open, relay},
};
use super::*;
use crate::cloud::socket::input;
use std::io::Cursor;

#[test]
#[ignore = "needs network access"]
fn a_wrong_key_is_refused_as_such() {
    let settings =
        settings(&std::env::var("HERDR_BOXD_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.into()));
    let key = secrecy::SecretString::from("not-a-real-key");
    let request = Request {
        settings: &settings,
        key: &key,
    };
    let error = request.machines().unwrap_err();
    eprintln!("wrong key: {error}");
    assert!(matches!(error, Error::Authentication), "{error:?}");
    let error = open(&settings, &key, "anything", "true")
        .and_then(|mut connection| {
            relay(
                &mut connection,
                input(Cursor::new(Vec::new())),
                &mut Vec::new(),
                &mut Vec::new(),
            )
        })
        .unwrap_err();
    eprintln!("wrong key on exec: {error}");
    assert!(matches!(error, Error::Authentication), "{error:?}");
}
