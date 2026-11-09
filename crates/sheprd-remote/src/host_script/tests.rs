// ABOUTME: Checks explicit host routes, development fences, and script bounds.
// ABOUTME: Uses only short local fixture scripts and never a personal daemon.
#![allow(clippy::unwrap_used)]

use super::*;

#[test]
fn hosts_follow_their_endpoint_target() {
    assert_eq!(
        Host::new(&ConnectTarget::Local).unwrap(),
        Host {
            route: Route::Local,
            session: None,
            development: false,
            env: Vec::new(),
        }
    );
    let ssh = Host::new(&ConnectTarget::Ssh {
        target: "me@box".into(),
        session: "work".into(),
    })
    .unwrap();
    assert_eq!(ssh.route, Route::Ssh("me@box".into()));
    assert!(ssh.prelude().contains("herdr --session 'work' \"$@\""));
    assert!(matches!(
        Host::new(&ConnectTarget::Socket("/tmp/s".into())),
        Err(Error::UnsupportedHost)
    ));
    // A `--dev` window's session lives in the development installation,
    // which `herdr --session` cannot name: its host still runs scripts,
    // but host-bound Herdr operations are refused there.
    let development = ConnectTarget::Session {
        name: "default".into(),
        development: true,
    };
    let host = Host::new(&development).unwrap();
    assert!(!host.scripts_herdr());
    assert!(!host.prelude().contains("herdr --session"));
    assert!(matches!(
        host_for(&development),
        Err(Error::DevelopmentSession)
    ));
    let release = ConnectTarget::Session {
        name: "default".into(),
        development: false,
    };
    assert!(Host::new(&release).unwrap().scripts_herdr());
    assert!(host_for(&release).is_ok());
}

/// Ordinary scripts still run on a development host; `herdr_cli` refuses,
/// so nothing reaches the release session of the same name.
#[test]
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn development_hosts_run_scripts_but_not_herdr() {
    let host = Host::new(&ConnectTarget::Session {
        name: "default".into(),
        development: true,
    })
    .unwrap();
    let cancelled = AtomicBool::new(false);
    assert_eq!(host.capture("printf ok", &cancelled).unwrap(), b"ok");
    let refused = host
        .capture(&Host::herdr_line(&["workspace", "list"]), &cancelled)
        .unwrap_err();
    assert!(
        matches!(
            &refused,
            sheprd_client::Error::ScriptExit { status, stderr }
                if status.code() == Some(97) && stderr.contains("--dev")
        ),
        "{refused:?}"
    );
}

#[test]
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn local_scripts_run_under_the_prelude() {
    let host = Host::new(&ConnectTarget::Local).unwrap();
    let cancelled = AtomicBool::new(false);
    let output = host.capture("printf '%s' \"$1-${PATH%%:*}\" || :\n", &cancelled);
    // `set -u` makes the unset positional an error: the prelude is active.
    assert!(matches!(
        output,
        Err(sheprd_client::Error::ScriptExit { .. })
    ));
    let mut output = Vec::new();
    host.stream(
        "cat\n",
        &b"through"[..],
        &mut output,
        ScriptLimits {
            output: 32,
            idle: QUERY_IDLE,
        },
        &cancelled,
    )
    .unwrap();
    assert_eq!(output, b"through");
}

#[test]
fn host_and_arguments_keep_their_exact_quoted_session() {
    let session = "work's session";
    let host = host_for(&ConnectTarget::Ssh {
        target: "someone@exact-host".into(),
        session: session.into(),
    })
    .unwrap();
    assert_eq!(host.route, Route::Ssh("someone@exact-host".into()));
    assert_eq!(host.session.as_deref(), Some(session));
    assert!(
        host.prelude()
            .contains("herdr --session 'work'\\''s session'")
    );
    assert_eq!(
        Host::herdr_line(&["pane", "move", "a'b", "target"]),
        "herdr_cli 'pane' 'move' 'a'\\''b' 'target'\n"
    );
    assert!(matches!(
        Host::new(&ConnectTarget::Wsl {
            distro: "fixture".into(),
            session: "work".into()
        }),
        Err(Error::UnsupportedHost)
    ));
}

#[test]
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn host_scripts_keep_cancellation_and_output_bounds() {
    let host = host_for(&ConnectTarget::Local).unwrap();
    assert!(matches!(
        host.capture("printf ignored", &AtomicBool::new(true)),
        Err(sheprd_client::Error::ScriptCancelled)
    ));
    let mut output = Vec::new();
    let result = host.stream(
        "printf abcd",
        &[][..],
        &mut output,
        ScriptLimits {
            output: 3,
            idle: QUERY_IDLE,
        },
        &AtomicBool::new(false),
    );
    assert!(matches!(
        result,
        Err(sheprd_client::Error::ScriptOutputLimit)
    ));
    assert!(output.len() <= 3);
}
