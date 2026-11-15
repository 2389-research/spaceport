use super::listening_ports::exit_status;
use super::*;
use crate::menu::devices::Error;

#[test]
fn every_devices_variant_has_a_message() {
    let detail = || "machine add: refused".to_owned();
    assert_localized(&[
        Error::Busy,
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
