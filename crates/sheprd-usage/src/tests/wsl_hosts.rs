use super::*;

#[cfg(windows)]
#[test]
fn a_name_that_is_not_a_distribution_starts_no_shell() {
    let error = crate::Shell::connect_wsl("a;b").err().unwrap();
    assert!(
        matches!(
            error,
            Error::Client(sheprd_client::Error::Wsl(
                sheprd_client::wsl::Error::InvalidDistro
            ))
        ),
        "{error:?}"
    );
}

#[cfg(not(windows))]
#[test]
fn a_distribution_is_only_reached_from_windows() {
    let error = crate::Shell::connect_wsl("Ubuntu").err().unwrap();
    assert!(matches!(error, Error::WslHostUnsupported), "{error:?}");
}
