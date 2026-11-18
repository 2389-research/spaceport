// ABOUTME: Bounds protocol negotiation with peers that keep changing their version.
// ABOUTME: Preserves the second refusal without making another connection.
use super::*;

#[test]
fn a_terminal_hello_changes_versions_at_most_once() {
    let (mut connect, peers) = streams(2);
    let mut attempts = 0;
    let herdr = thread::spawn(move || {
        for (mut peer, version) in peers.into_iter().zip([23, 24]) {
            receive(&mut peer);
            send(&mut peer, welcome(version, Some("version changed")));
        }
    });
    let result = TerminalAttach::open_with(
        || {
            attempts += 1;
            connect()
        },
        "term_1",
        80,
        24,
    );
    herdr.join().unwrap();
    assert!(
        matches!(result, Err(Error::Rejected { version: 24, ref reason }) if reason == "version changed"),
        "the second refusal must end negotiation: {result:?}"
    );
    assert_eq!(attempts, 2);
}
