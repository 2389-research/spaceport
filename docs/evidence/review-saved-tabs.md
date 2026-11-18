# Saved tab recovery

Review finding: `fzwm`, under `9s44`.

## Cause

Both saved-state loaders converted every read or parse failure into empty state.
They started replacement writers even after failures.
The tab loader also reset the ID counter without inspecting saved group references.
A later save could destroy rejected bytes and reuse a referenced tab ID.

## Change

Rejected bytes survive in unique sibling files named `<file>.rejected-<random>`.
macOS fixtures verify backup mode `0600`.
The backup file is synced before the replacement writer can start.
Unix also syncs the parent directory.
Recovery never prunes or replaces earlier backups.

Parseable files recover each valid entry with the existing typed validators.
Existing tab and layout entry validators remain strict.
The tab counter reserves rejected tab IDs and saved counters.
It also reserves page IDs in group picks, owned tabs, and legacy hidden tabs.
This scan includes entries that fail typed layout validation.

Failed reads or failed backup writes prevent replacement writer startup.
Unreadable layout envelopes also block tab allocation and saving.
Those layout files remain unchanged across restarts until repaired.
Existing readable tabs remain available.
Layouts retain a `next_tab_id` reservation floor beside the saved layout list.
Every layout write preserves that floor, even after removing its last referenced entry.
The floor and list share one atomic replacement, so a failed write preserves the original references.
Store merges this floor with current references and its own allocation counter.

Core tab creation reserves an identity and capacity slot before queueing or connecting.
The FIFO owns the reservation through submission and placement.
Unrelated tab opens account for reserved slots.
Successful placement consumes the original slot instead of allocating again.
Queue refusal, stale targets, failed workers, rejected requests, and window drop release their slots.
Hub failure clears queued requests but retains submitted work until its outcome or owner drops.
Unknown outcomes never trigger replay or terminal termination.
If a replaced Store prevents placement, the warning reports the terminal ID and confirmed creation.

Torn tab JSON cannot recover individual entries automatically.
Its exact bytes remain in the backup before a later save replaces the live file.
Layout references still reserve their IDs.

## Verification

All fixtures use temporary files and synthetic identities.
No personal state, daemon, or credentials are accessed.

- Initial regression run: six expected failures from real file recovery tests.
- Restart regression: failed because the layout writer remained enabled; then fixed.
- UI regression: observed the incorrect `Too many tabs are open` flash from the Devices action.
- Core preflight regression: observed no immediate refusal before the runtime connection was queued.
- Independent review exposed lost reservation floors after layout-only writes and insufficient capacity admission.
- Both review regressions failed before their fixes: floor `0` instead of `301`, and no exhausted-ID refusal.
- Updated recovery filter: 27 passed, including layout-only restart, saturated floors, and failed floor writes.
- Full browser filter: 122 passed.
- Full Core filter: 135 passed; eight existing live tests need an explicit Core binary and remained ignored.
- Core fixtures verify runtime-refusal cleanup and successful placement after unrelated opens fill capacity.
- A replaced-Store fixture verifies the created terminal ID reaches the warning without replay.
- Scoped `rustfmt --edition 2024 --check` and `git diff --check`: passed.

The recovery test command was:

```sh
/usr/bin/lockf -k /tmp/spaceport-qnz9/cargo.lock env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/Users/harper/Public/src/2389/spaceport/.worktrees/upstream-fixes/target cargo test --locked -p sheprd-gpui --bin spaceport recovery
```

The final browser and Core checks used the same lock and environment with these Cargo arguments:

```sh
cargo test --locked -p sheprd-gpui --bin spaceport browser::
cargo test --locked -p sheprd-gpui --bin spaceport core_terminals::
```

Logs live under `/tmp/spaceport-qnz9/` for this local session:
`fzwm-red.log`, `fzwm-green.log`, `fzwm-restart-red.log`, `fzwm-ui-red.log`, and `fzwm-core-preflight-red.log`.
The root task's integrated run is `/tmp/spaceport-9s44-recovery-green.log`.
Its final run, including the Core preflight guard, is `/tmp/spaceport-9s44-recovery-final.log`.
Follow-up logs are `fzwm-reservations-red.log`, `fzwm-reservations-green.log`, `fzwm-reservation-slots.log`,
`fzwm-browser-final.log`, and `fzwm-core-final.log` under `/tmp/spaceport-qnz9/`.

The debug test link emits the existing macOS `__eh_frame` size warning.
Cargo also reports the existing `block` dependency future-compatibility notice.

Full CI, Windows execution, and native desktop QA remain deferred.
Headless GPUI checks exercise the action and warning state, not native OS drawing.

Independent follow-up review approved the durable reservation floor and Core FIFO ownership fixes.
It inspected restart behavior, admission, completion, refusal, cancellation, and owner-drop paths.
No required defects remained in that scope.
