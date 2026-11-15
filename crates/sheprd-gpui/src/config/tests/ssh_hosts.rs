use super::*;

#[test]
fn hosts_skip_host_key_verification_only_when_listed() -> anyhow::Result<()> {
    let config = Config::parse("[ssh]\nskip_host_key_verification = ['dev.box.coder']")?;
    assert!(config.unknown_keys.is_empty(), "{:?}", config.unknown_keys);
    assert!(
        config
            .ssh
            .skip_host_key_verification
            .contains("dev.box.coder")
    );
    assert!(
        !config
            .ssh
            .skip_host_key_verification
            .contains("other.coder")
    );
    // Nothing is skipped by default.
    let config = Config::parse("")?;
    assert!(config.ssh.skip_host_key_verification.is_empty());
    assert!(
        !config
            .ssh
            .skip_host_key_verification
            .contains("dev.box.coder")
    );
    Ok(())
}

#[test]
fn a_list_that_names_no_ssh_target_is_refused() -> anyhow::Result<()> {
    for text in [
        "[ssh]\nskip_host_key_verification = ['-oProxyCommand=x']",
        "[ssh]\nskip_host_key_verification = ['two words']",
        "[ssh]\nskip_host_key_verification = ['']",
    ] {
        let error = Config::parse(text).err().context("accepted")?;
        assert!(
            matches!(
                error,
                crate::Error::SshHosts(crate::config::ssh_hosts::Error::InvalidTarget(_))
            ),
            "{text:?} gave {error:?}"
        );
    }
    let many = (0..=sheprd_client::MAX_UNVERIFIED_HOSTS)
        .map(|index| format!("'host-{index}'"))
        .collect::<Vec<_>>()
        .join(", ");
    let error = Config::parse(&format!("[ssh]\nskip_host_key_verification = [{many}]"))
        .err()
        .context("accepted")?;
    assert!(
        matches!(
            error,
            crate::Error::SshHosts(crate::config::ssh_hosts::Error::TooMany(_))
        ),
        "{error:?}"
    );
    Ok(())
}
