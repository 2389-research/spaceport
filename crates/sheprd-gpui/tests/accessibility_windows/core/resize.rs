// ABOUTME: Qualifies stock prefix resizing through two real Core terminal panes.
// ABOUTME: Checks native bounds and exact runtime identities without ending either terminal.
use super::{Attach, Duration, Fixture, Instant, require_runtime_output, thread};

#[test]
#[ignore = "requires Windows desktop plus explicitly selected Herdr and Core fixture binaries"]
fn native_core_prefix_resize_moves_and_restores_bounds_without_ending_either_terminal() {
    let mut fixture = Fixture::start();
    let original = fixture.open_terminal();
    fixture.running.control(&["run", "split_right"]);
    let deadline = Instant::now() + Duration::from_secs(15);
    let right = loop {
        let terminals = fixture.api().list(false).unwrap().terminals;
        if terminals.len() == 2
            && terminals
                .iter()
                .any(|terminal| terminal.address == original)
        {
            break terminals
                .into_iter()
                .find(|terminal| terminal.address != original)
                .unwrap()
                .address;
        }
        assert!(
            Instant::now() < deadline,
            "Core split did not preserve its source terminal"
        );
        thread::sleep(Duration::from_millis(20));
    };
    fixture
        .running
        .wait_tree("two native Core terminal providers", |tree| {
            tree.matches("Document 'Terminal' [Value, Text]").count() == 2
        });
    let info = fixture.api().terminal_get(&right).unwrap();
    let observed = fixture
        .api()
        .attach(&Attach {
            cols: info.cols,
            rows: info.rows,
            terminal: &right,
            observe: true,
            ..Attach::default()
        })
        .unwrap();
    let capture = fixture.running.sandbox.dir.join("core-resize.log");
    let command = fixture.terminal_helper("CoreResize", "core-resize.log");
    let output = fixture
        .running
        .command_output(command, &capture, Duration::from_secs(30))
        .unwrap();
    assert!(output.contains("TERMINAL_RESIZE native=true prefix=true left=true right=true restored=true released=true"), "{output}");
    require_runtime_output(&observed, b"SP_RESIZE_7");
    let mut expected = vec![original, right];
    expected.sort();
    let mut after = fixture
        .api()
        .list(false)
        .unwrap()
        .terminals
        .into_iter()
        .map(|terminal| terminal.address)
        .collect::<Vec<_>>();
    after.sort();
    assert_eq!(
        after, expected,
        "Core resize changed runtime terminal identities"
    );
    eprintln!(
        "CORE_RESIZE native=true prefix=true left=true right=true restored=true runtime_marker=true stable_terminal_ids=true"
    );
}
