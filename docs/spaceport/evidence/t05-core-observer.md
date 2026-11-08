# T05/A13: private Core observer input and size

At source `5a6d8cbb`, a new opt-in live test attaches a `Role::Observer` to a
terminal owned by a private Core fixture. The observer sends text, Enter,
resize and focus requests through the retained `CoreBackend`. Core's terminal
dimensions remain at their original values. A second, active attachment sends
a different line; the fixture shell prints only that active line. The test
checks rendered cells, not a mocked answer, and it ends only its own terminal
and Core child through the existing private sandbox.

`HERDR_CORE_TEST_BINARY=/tmp/spaceport-t00-target/debug/herdr-core
CARGO_TARGET_DIR=/tmp/spaceport-t00-target CARGO_INCREMENTAL=0 cargo +1.99.0
test --locked -p sheprd-core-client --lib
tests::live::observer::observer_input_and_resize_leave_the_program_for_its_active_view
-- --ignored --exact --nocapture`: 1 passed on macOS. The fixture Core is the
pinned 0.1.18 binary identified in `evidence/t05-core-lifecycle.md`. No
personal socket, agent, home, or network machine was used.

The default `sheprd-core-client` library suite passed 56 tests with nine
opt-in tests ignored. `just format-check`, `just check-file-size`,
`just check-sorted`, and the Spaceport package check and 21 tool tests passed.

This adds a real Core regression to the existing pure backend observer tests.
At that point, confirmed RESIZE ordering with subsequent paint/data, remote
observers, and native view behavior remained untested. The follow-on tests the
local ordering path; A13 remains `not_run` as a complete acceptance case until
remote, older-Core, and native view behavior is proved. It does not establish
A11 or A12. T05 stays `in_progress`.

The pinned Core `schema/README.md:84–101` defines an optional
`resize_notifications` attach request and a confirmed reply. A capable observer
must apply incoming RESIZE before later DATA; an older path uses separate size
tracking. At source `64e9cb05`, this client omitted that request and reply
field (`wire.rs::Attach` and `Attached`), ignored stream RESIZE in
`connection.rs::Worker::dispatch`, and polled `terminal.list` every 500 ms in
`backend/terminal.rs`. Confirmed ordered resize was an **internal client
implementation gap**, not evidence that Core lacked the route. The subsequent
`eb092a10` cut negotiates the feature, preserves RESIZE/DATA order in the
bounded stream inbox, applies size before paint, and retains a polling
fallback for an unconfirmed path. See the confirmed resize section of
[t05-core-lifecycle.md](t05-core-lifecycle.md). No Core source was changed.
