// ABOUTME: Tests agent command selection before any process starts.
// ABOUTME: Preserves explicit custom paths while refusing unresolved bare names.
#![allow(clippy::unwrap_used)]
use super::*;

fn options(program: &str) -> Options {
    let crate::cli::Parsed::Run(options) =
        crate::cli::parse(["--", program].map(Into::into)).unwrap()
    else {
        panic!("expected command options");
    };
    options
}

#[test]
fn a_missing_custom_bare_name_is_refused_before_spawn() {
    let root = tempfile::tempdir().unwrap();
    let name = format!("spaceport-fixture-{}", uuid::Uuid::new_v4());
    assert!(agent(&options(&name), root.path()).is_err());
}

#[test]
fn an_explicit_custom_relative_path_stays_explicit() {
    let root = tempfile::tempdir().unwrap();
    let selected = agent(&options("./fixture-agent"), root.path()).unwrap();
    assert_eq!(selected.program, "./fixture-agent");
}
