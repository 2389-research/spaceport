//! Every variant of this crate's failures words itself for a window.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::{Error, host_script, setup};
use spaceport_i18n::assertions::{assert_localized, assert_verbatim};

fn io() -> std::io::Error {
    std::io::Error::other("herdr: not found")
}

/// A process that exited with status 1, as the system describes it.
fn exit_status() -> std::process::ExitStatus {
    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(windows)]
    use std::os::windows::process::ExitStatusExt;
    #[cfg(unix)]
    let code = 1 << 8;
    #[cfg(windows)]
    let code = 1;
    std::process::ExitStatus::from_raw(code)
}

#[test]
fn every_host_refusal_has_a_message() {
    assert_localized(&[
        host_script::Error::UnsupportedHost,
        host_script::Error::DevelopmentSession,
    ]);
    assert_localized(&[Error::HostScript(host_script::Error::DevelopmentSession)]);
}

#[test]
fn every_device_setup_variant_has_a_message() {
    let detail = || "machine add: refused".to_owned();
    assert_localized(&[
        setup::Error::Busy,
        // Delegated: the client's own mapping of an unreachable host.
        setup::Error::Client(spaceport_client::Error::Io(io())),
        setup::Error::InvalidLabel,
        setup::Error::NoProfile,
        setup::Error::Remove {
            status: exit_status(),
            detail: String::new(),
        },
        setup::Error::Remove {
            status: exit_status(),
            detail: detail(),
        },
        setup::Error::Rename {
            status: exit_status(),
            detail: String::new(),
        },
        setup::Error::Rename {
            status: exit_status(),
            detail: detail(),
        },
        setup::Error::SetupTimeout,
    ]);
    assert_localized(&[Error::Setup(setup::Error::SetupTimeout)]);
}

#[test]
fn an_unstarted_process_shows_the_system_text() {
    assert_verbatim(&[setup::Error::Io(io())]);
}
