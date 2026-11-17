# Explicit state machines implementation plan

> Use isolated task worktrees, TDD, and independent code review for each child.
> Use the parallel-agents workflow only for disjoint files and worktrees.

**Goal:** Complete Kata qnz9 and all 15 child issues.

**Architecture:** Give each lifecycle one typed state and one transition owner.
Keep side effects at existing boundaries.
Preserve wire formats, saved files, and CLI schemas.
Apply the behavior corrections requested by each child issue.

**Stack:** Rust 1.99.0, GPUI 0.3.10, and the existing synchronous workers.

## Global constraints

- Read the current child issue before changing its component.
- Pin existing behavior and prove each reported regression before changing the state representation.
- Use one branch and one pull request per machine.
- Keep handwritten source files below 1,000 lines.
- Preserve explicit runtime targets, cancellation, cleanup, and bounded resource use.
- Keep desktop runtime installation, agent recovery, and excluded features out of scope.
- Keep shared wire and saved representations unchanged.
- Prefer focused tests over full local workspace CI.
- Run required quick checks before commits and pushes.
- Run enabled CI gates before merging.
- Review each diff independently before merging.
- Close a child only after its source, tests, documentation, and merge are verified.
- Add `docs/state-machines.md` with the first conversion and extend it per conversion.
- Keep shared documentation and integration under the coordinator's ownership.
- Preserve all unrelated worktrees and untracked files.

## Per-machine delivery steps

Apply these steps separately to every row below.

- [ ] Inspect the child issue, current fields, callers, tests, and all state writes.
- [ ] Add a regression or characterization test at the existing test boundary.
- [ ] Observe the expected failure, or record why baseline failure cannot be reproduced.
- [ ] Introduce the lifecycle enum and its transition function.
- [ ] Route all lifecycle writes through that owner and remove obsolete flags.
- [ ] Verify the requested behavior and adverse paths.
- [ ] Add the machine's states, transitions, owner, and tests to the registry.
- [ ] Run focused tests, applicable native checks, and quick repository checks.
- [ ] Obtain independent review and resolve material findings.
- [ ] Commit, push, open the machine's PR, and pass enabled CI.
- [ ] Merge the PR and close the child with typed evidence.

## Task order and ownership

The Kata issue bodies are the detailed acceptance contracts.
The initial audit used `b2ca7c2b1`; implementation starts from `724957c05`.
Recheck the code instead of trusting historical line numbers.

| Order | Kata | Deliverable | Source and test owners | Dependencies |
| --- | --- | --- | --- | --- |
| 1 | xtjm | Fence chat upload and browse completions to their original conversation | `chat/window.rs`, `chat/chats.rs`, `chat/pasted.rs`, `chat/tests/` | None |
| 2 | y0p7 | Keep newer theme and layout intents through saving and quitting | `settings_window.rs`, `settings_window/persistence.rs`, `themes.rs`, `layouts.rs`, `sidebar.rs`, settings tests | None |
| 3 | ha8y | Preserve failed Codex handshakes and refuse invalid Pi opening | `sheprd-acp/src/codex/bridge.rs`, `pi/bridge.rs`, bridge tests | None |
| 4 | ed3g | Derive tool liveness once for every renderer after turn and session end | `sheprd-chat-protocol`, ACP projection/TUI, GPUI chat renderers | xtjm |
| 5 | 1w6g | Queue and drain every Core terminal spawn request | `core_terminals.rs`, spawn tests | None |
| 6 | dre7 | Retain and retry an unacknowledged Herdr activity report | `sheprd-acp/src/herdr.rs`, reporter tests | None |
| 7 | 77dt | Store the endpoint link phase and reset retargeted diagnostics | `endpoint.rs`, `endpoint/`, lifecycle tests | None |
| 8 | 674p | Close late Core stream answers and fence lost-call registration | `sheprd-core-client/src/connection.rs`, connection tests | None |
| 9 | t9fk | Match tunnel completions to the exact attempt | `listening_ports/tunnel.rs`, tunnel tests | None |
| 10 | fj07 | Consume Core start intent within its owning connection attempt | `core_terminals/hub.rs`, hub tests | 1w6g |
| 11 | 7dc6 | Keep retryable activation failures on their selected host | `state.rs`, `window/commands.rs`, endpoint polling and lifecycle tests | 77dt |
| 12 | hmqh | Hold successful transfer pastes until their terminal is ready | `window/transfers.rs`, transfer tests | None |
| 13 | d84r | Count written prompt deliveries and report exhausted retry honestly | `chat/session.rs`, `chat/chats.rs`, session/link tests | xtjm, ed3g |
| 14 | rd2q | Hold Claude, Codex, and Pi turns in explicit states | ACP bridge modules and their turn/cancellation tests | ha8y, dre7 |
| 15 | z1tp | Give status notices a cause, expiry, and one clearing path | `window.rs`, `state.rs`, notice writers and status tests | All other UI conversions |

## Verification contract

Run the matching crate and topic tests with `cargo test --locked`.
Use `--features integration-test --bin spaceport` for headless GPUI topic tests.
Reuse the cleared-environment process sandbox for executable and native checks.
Select explicit fixture runtimes and never use personal daemon state.
Use corresponding platform CI for platform gates.

Run `just format-check check-file-size check-sorted check-i18n` for application changes.
Run scoped Clippy with `-D warnings` where applicable.
Record existing dependency warnings separately from newly introduced warnings.

## Completion audit

Kata remains the authoritative issue-status ledger.
Record each child's PR and exact verification in its own comments and close evidence.
Keep the coordinator's resumable ledger under `.superpowers/qnz9/`.
Before closing qnz9, inspect every child and every merged source transition owner.
Verify the registry covers all 15 machines and their tests.
Do not close the parent while any required child or gate remains open.
