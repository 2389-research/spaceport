use super::*;

#[test]
fn retired_github_account_settings_are_rejected() {
    for source in [
        "[github]\noauth_client_id = 'Iv1.fixture'",
        "[github]\nallow_plaintext_credentials = true",
    ] {
        let error = Config::parse(source).expect_err("old table must fail");
        assert!(matches!(
            error,
            crate::Error::Settings(Error::RemovedGitHub)
        ));
    }
}
