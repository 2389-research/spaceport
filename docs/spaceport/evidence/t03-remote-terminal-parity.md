# T03/T05 remote Core terminal view

The pinned Core routes `terminal.run` with `on` through a saved machine's
permanent `Machine.id` as well as its mutable name (`remote.rs:744–779`). Its
`terminal.list` aggregates remote records with the saved **name** as the
address prefix (`api.rs:238–281`, `remote.rs:1000–1033`). A terminal method,
including `terminal.attach`, resolves an ID-qualified prefix; the Core client
already passes `Spawn.machine` as `Run.on` and `CoreTerminal::attach` runs on a
worker. No new RPC, SSH manager, second subscription, or remote document write
is needed.

The private two-Core `run_and_attach_route_to_selected_remote_session_by_id`
test uses a test-only `HC_SSH` relay that accepts only a `target` session. It
adds the target Core to a private hub, runs a shell with `on=Machine.id`, checks
the remote PID and the hub's aggregated list, renames the machine, then
attaches through `Machine.id/terminal_id` and reads the same terminal output.
It checks `terminal.get` through the ID route and that the hub lists one
process. Both exact private Core children are stopped and reaped. The test
passed 1/1 on macOS with pinned `/tmp/spaceport-t00-target/debug/herdr-core`.
The command used `HERDR_CORE_TEST_BINARY=/tmp/spaceport-t00-target/debug/herdr-core`,
`CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/spaceport-t03-parity-client-target`,
and `cargo +1.99.0 test --locked -p sheprd-core-client --lib
run_and_attach_route_to_selected_remote_session_by_id -- --ignored`.

The first app action draft tried to open a hub group member in a native tab.
A private two-Core GUI experiment created a remote member in the hub document,
ran one remote terminal and selected the group. The member row rendered, but
the draft could not open a tab: `SheprdWindow::browser_key()` returned `None`
in a Core-only window. It derives `(Scope, workspace_id)` from Herdr 1's
`live.snapshot.focused_workspace_id` (`browser/view.rs:178–181`). The tab
store and Core terminal view sync both require that key (`browser/store.rs`,
`core_terminals/tabs.rs:30–43`). The browser's no-workspace fallback group is
only a render slot (`browser/groups_view.rs:109–117`); it supplies no stable
Core presentation group key. A selected Core group UUID and permanent hub
Core ID exist, but binding them to a new `Store` scope and window layout is a
separate ownership/restore decision. The draft UI code and failing GUI test
were removed, so the remote member remains visibly unavailable rather than
becoming a clickable no-op.
The draft GUI case failed after the group and exact remote member row rendered;
the test's `browser_key()` check had no workspace to name. This is a failed
exploration result, not a passed acceptance check.

The next app slice must define a Core-backed tab/container key from the hub's
versioned group UUID and local Core ID, plus selection, tab rendering and
restore when there is no Herdr 1 workspace. It can then use the proved
`Machine.id/terminal_id` attach target while mapping Core's mutable list
address only for current display facts. It must keep the hub document as the
single grouping authority. This cut makes no production UI or backend change.
Final local checks are the focused Core client case, `just format-check`,
`just check-file-size`, `git diff --check`, and package `spec.py check`.

T03 and T05 remain `in_progress`. A08, A09 and A53 remain `not_run`: this slice
does not offer grouped launch or document assignment after a new run, prove
restore across app restart, or exercise the full remote document missed/epoch
scenario in native UI. Remote terminal creation from the selected Devices
machine remains an explicit UI gap. The existing `sheprd tab focus` command
can focus an open view; adding a command that repeats Core's own terminal
run/attach CLI would violate the local CLI contract. Native keyboard,
accessibility, resize, and visual QA remain unrun.
