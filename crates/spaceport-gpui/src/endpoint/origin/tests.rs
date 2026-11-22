use super::*;

fn ssh(target: &str) -> Origin {
    Origin::of(&ConnectTarget::Ssh {
        target: target.into(),
        session: "default".into(),
    })
}

#[test]
fn local_targets_are_this_machine() {
    for target in [
        ConnectTarget::Local,
        ConnectTarget::Session {
            name: "work".into(),
            development: false,
        },
        ConnectTarget::Socket("/tmp/herdr.sock".into()),
    ] {
        assert_eq!(Origin::of(&target), Origin::Local, "{target:?}");
    }
    assert_eq!(
        Origin::of(&ConnectTarget::Wsl {
            distro: "Ubuntu".into(),
            session: "default".into(),
        }),
        Origin::Wsl
    );
}

#[test]
fn tailnet_names_and_addresses_are_tailscale() {
    for target in [
        "penso@100.84.59.116",
        "100.64.0.1",
        "100.127.255.255:22",
        "m4max.tail1234.ts.net",
        "user@M4MAX.TAIL1234.TS.NET.",
        "ssh://me@[fd7a:115c:a1e0::1]:22",
        "fd7a:115c:a1e0:ab12::1",
    ] {
        assert_eq!(ssh(target), Origin::Tailscale, "{target}");
    }
    // Neighbours of the ranges, and look-alike names, are plain SSH.
    for target in [
        "100.63.255.255",
        "100.128.0.1",
        "10.0.0.5",
        "fd7a:115c:a1e1::1",
        "ts.net.example.com",
        "workspace.owner.coder",
        "build-box",
        "user@example.com",
    ] {
        assert_eq!(ssh(target), Origin::Ssh, "{target}");
    }
}

#[test]
fn every_mark_is_an_embedded_icon() {
    use gpui::AssetSource;
    for origin in [Origin::Local, Origin::Ssh, Origin::Tailscale, Origin::Wsl] {
        assert!(
            crate::icons::Icons
                .load(origin.icon())
                .ok()
                .flatten()
                .is_some(),
            "{origin:?}"
        );
        assert!(!origin.describe().is_empty());
    }
}

#[test]
fn host_names_drop_user_port_scheme_and_brackets() {
    assert_eq!(host_name(" ssh://Me@Host.Example.: "), "host.example");
    assert_eq!(host_name("me@host:2222"), "host");
    assert_eq!(host_name("[::1]:22"), "::1");
    assert_eq!(host_name("fe80::1"), "fe80::1");
}
