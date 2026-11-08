# Orchestration command and control route

At `38f808e3`, `sheprd orchestrator` parsed `open` and local `items|issues|prs|tasks` reads. The open command sent `TabOp::Orchestrator` through the desktop control socket, whose server switched to the caller's workspace and opened the Issues section. The item commands read the orchestrator's local SQLite store without an app. `docs/cli.md`, the app README, `docs/orchestrator.md`, help text and a source comment still advertised the command. The repository's managed skill had no orchestrator entry.

This cut removes the whole `sheprd orchestrator` family: parser/help/dispatch, the local item reader and its command-only errors, and the control wire/server arrival operation. Protocol version 21 becomes 22 because an old `{"method":"tab","caller":{"workspace_id":"w4"},"op":{"op":"orchestrator"}}` request is now refused by strict decoding. The retained `devices` command keeps its `--window` and `--json` parsing, and its wire request preserves exact daemon socket, window and workspace fields. `TabKind::Orchestrator` remains as metadata for native tabs that still exist; this does not keep the deleted open operation. The native Issues UI, `OpenOrchestrator` command, `ui panel issues|tasks`, orchestrator backend/worker and stored tabs remain for separate removal units. No Core or user environment changed.

- Red: focused `sheprd-control-protocol` test failed because the old orchestrator tab operation decoded; focused `sheprd-gpui --bin sheprd` parser test failed because `orchestrator` still returned `ControlCommand::Orchestrator`.
- Green: the same focused wire and parser tests each passed 1/1 after removal; the control-protocol library suite passed 56/56. The actual Cargo-built desktop executable passed a private-state CLI scenario: retired command shapes, including `orchestrator --help`, exit as usage errors before app contact; top-level help omits the family; retained `devices --json --window 7` attempts only the isolated app target and reports `not_running`. A focused headless control-server test passed for a retained Devices request with an unknown workspace. No GUI, daemon, service or personal socket was started.
- `just check-i18n` passed: 20 script tests and 5,160 used messages in 269 files.
- `just format-check`, `just check-file-size`, `just check-sorted`, and `git diff --check` passed. Package-root `uv run --with jsonschema==4.26.0 python tools/spec.py check` passed specification integrity checks; `tools/test_spec.py` passed 21 tests. Cargo emitted the existing macOS `__eh_frame` linker and `block v0.1.6` future-compatibility notices. Full native UI and cross-platform suites were not run.

T04 and RM-DISPATCH stay `in_progress`. A30 stays `not_run`: native orchestration UI/backend and other excluded routes still exist, and no fake provider endpoint or installed service inventory ran.

## Merged verification

At `7b802b30`, the route is combined with the legacy plugin Settings and
Boxd Dispatch cuts. Protocol version 22 passed 56/56 library tests. The
focused parser test and real desktop CLI private-state scenario passed 1/1
each; retained server tab tests passed 8/8. Default and all-feature app checks
passed without new Rust code warnings. Format, file-size, sorted-list and
i18n checks passed (5,138 used messages in 268 files); the package integrity
check and 21 tool tests passed. Native orchestration UI and the backend
remain, so A30 stays `not_run`.
