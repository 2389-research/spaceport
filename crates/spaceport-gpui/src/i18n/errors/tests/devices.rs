use super::listening_ports::exit_status;
use super::*;
use spaceport_remote::setup::Error;

fn io() -> std::io::Error {
    std::io::Error::other("herdr: not found")
}

#[test]
fn every_devices_variant_has_a_message() {
    let detail = || "machine add: refused".to_owned();
    assert_localized(&[
        Error::Busy,
        // Delegated: the client's own mapping of an unreachable host.
        Error::Client(spaceport_client::Error::Io(io())),
        Error::InvalidLabel,
        Error::NoProfile,
        Error::Remove {
            status: exit_status(),
            detail: String::new(),
        },
        Error::Remove {
            status: exit_status(),
            detail: detail(),
        },
        Error::Rename {
            status: exit_status(),
            detail: String::new(),
        },
        Error::Rename {
            status: exit_status(),
            detail: detail(),
        },
        Error::SetupTimeout,
    ]);
}

#[test]
fn an_unstarted_process_shows_the_system_text() {
    assert_verbatim(&[Error::Io(io())]);
}
