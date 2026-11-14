// ABOUTME: Tests the observation CLI's three bounded read-only requests.
// ABOUTME: Keeps common window and JSON flags on every observation operation.
use super::*;
use sheprd_control_protocol::protocol::observation::{Bounds, Operation, Target};

#[test]
fn observation_commands_parse_bounds_targets_and_common_flags() {
    let invocation = control::parse(
        "observation".into(),
        [
            "terminal",
            "source-1",
            "--offset",
            "4",
            "--limit",
            "9",
            "--max-bytes",
            "128",
            "--window",
            "7",
            "--json",
        ]
        .into_iter()
        .map(OsString::from),
    )
    .unwrap();
    assert_eq!(invocation.window, Some(7));
    assert!(invocation.json);
    assert_eq!(
        invocation.command,
        control::ControlCommand::Observation(Operation::Terminal {
            target: Target("source-1".into()),
            bounds: Bounds {
                offset: 4,
                limit: 9,
                max_bytes: 128
            }
        })
    );
    for args in [
        vec!["sessions", "--limit", "0"],
        vec!["terminal", "/tmp/socket"],
        vec!["shell", "x"],
    ] {
        assert!(
            control::parse("observation".into(), args.into_iter().map(OsString::from)).is_err()
        );
    }
}
