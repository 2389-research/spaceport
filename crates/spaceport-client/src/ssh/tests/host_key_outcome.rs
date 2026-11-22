//! What a bridge reports about the host key `ssh` was and was not asked to
//! check, so a connection to a host the user exempted can say so.
use super::*;

const COMPATIBLE: &str = r#"{"endpoint_protocol_generation":1,"endpoint_capabilities":["surface_interest","presentation_effects_fence","health_check"]}"#;

/// Stands in for `ssh` running the bridge script: reports a compatible client
/// with its ready line, then reads the choice the handshake sends.
#[cfg(unix)]
fn stand_in_ssh() -> Command {
    let mut command = Command::new("/bin/sh");
    command.args([
        "-c",
        &format!(
            "printf '%s\\n' '{COMPATIBLE}' 'herdr-remote-output-ready:1'; IFS= read -r choice || exit 1"
        ),
    ]);
    command
}

#[cfg(unix)]
#[test]
fn a_bridge_to_a_host_whose_key_is_not_checked_reports_it_unverified() {
    let target = "outcome-unverified.test";
    let stop = AtomicBool::new(false);
    let reported = crate::tests::while_unverified(target, || {
        let host_keys = HostKeys::of(target);
        // One resolved choice builds the child `ssh` would run and describes it.
        assert!(
            command_with(host_keys, target, "true")
                .get_args()
                .any(|arg| arg == "StrictHostKeyChecking=no")
        );
        let Ok(started) = start_bridge(host_keys, stand_in_ssh(), &stop, handshake) else {
            panic!("the bridge did not start");
        };
        let Ok(Connected { host_keys, .. }) = started else {
            panic!("the bridge closed instead of answering");
        };
        host_keys
    });
    assert_eq!(reported, HostKeys::Skip);
}

#[cfg(unix)]
#[test]
fn a_bridge_to_an_ordinary_host_reports_its_key_verified() {
    let stop = AtomicBool::new(false);
    let Ok(started) = start_bridge(
        HostKeys::of("outcome-verified.test"),
        stand_in_ssh(),
        &stop,
        handshake,
    ) else {
        panic!("the bridge did not start");
    };
    let Ok(Connected { host_keys, .. }) = started else {
        panic!("the bridge closed instead of answering");
    };
    assert_eq!(host_keys, HostKeys::Verify);
}

#[cfg(unix)]
#[test]
fn a_refused_bridge_reports_no_connection_to_mark() {
    // A skipped host whose `ssh` still refused: nothing connected, so there is
    // no connection to describe.
    let mut command = Command::new("/bin/sh");
    command.args([
        "-c",
        "printf '%s\\n' 'Host key verification failed.' >&2; exit 255",
    ]);
    let Ok(started) = start_bridge(HostKeys::Skip, command, &AtomicBool::new(false), handshake)
    else {
        panic!("the bridge did not start");
    };
    let Err(closed) = started else {
        panic!("a failed bridge was accepted");
    };
    assert_eq!(closed.code, Some(255));
    assert_eq!(closed.failure, SshFailure::HostKey);
}
