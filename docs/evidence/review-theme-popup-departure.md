<!-- ABOUTME: Records theme-picker departure, native-field disposal, and retained write ownership. -->
<!-- ABOUTME: Separates actual headless regression evidence from pending native qualification. -->

# Theme picker departure

Date: 2026-11-17. Scope: the global Themes popup under `ydr4` and `9s44`.
Device Theme belongs to the separate host-menu model and remains outside this record.
Doctor Biz authorized implementation and validation.
This owner made no commit, push, release, or Kata change.

## Ownership and behavior

ThemePicker retains discovery, preview, accepted-save, and favorite-write accounting after dismissal.
An optional Input owns only the search entity and its subscription.
MenuState releases Input after the popup shell captures its departure data.
Reopening creates a fresh native field.
Search events must name that current field and the current Themes page.
Old queued events cannot replace a reopened query.

Accepted theme disk writes still block cancellation and modal replacement until their result is reconciled.
Unaccepted preview loads keep their existing session/request fence.
Discovery remains single-flight across reopening.
Accepted favorite writes can finish after the native field drops.
The retained owner reconciles their result and queued edits.

The outgoing Paint contains visible row data, field paint, counts, status text, and plain scroll offsets.
It retains no Entity, subscription, input handler, focus handle, or Task.
The live and outgoing header, row, star, and status content share their templates.
Outgoing rows, stars, and the close button register no mutation handlers.
The frozen list and status clip at their captured offsets without retaining scroll handles.

## Evidence

Two actual window regressions failed before implementation.
Dismissal retained the original search entity.
Changing retained live results removed the outgoing scrolled rows.
Log: `/tmp/spaceport-qnz9/ydr4-theme-departure-red.log`.
The final weak-lifetime assertion crosses a draw boundary so GPUI releases its previous input handler.

The first coherent Themes implementation passed 20 tests.
Log: `/tmp/spaceport-qnz9/ydr4-themes-green.log`.
Two additional lifecycle cases then passed with the full 22-test scope:

- Old search events after closing and reopening preserve the new empty query.
- An accepted favorite callback writes an actual private temporary file after Input drops.

The private-file test uses JSON at the callback boundary.
It proves callback lifetime and reconciliation, not the native configuration format or its locking.
Existing favorite tests also use a stand-in in-memory file; those cases alone do not qualify disk persistence.
The new departure fixture suppresses personal theme discovery and uses built-in palettes.
It never writes personal configuration.

All 22 theme tests passed with no failures or ignored cases.
Log: `/tmp/spaceport-qnz9/ydr4-themes-final-runtime.log`.
The artifact was freshly built by the shared `2njz-complete-build.log` run.
An earlier final build stopped on concurrent ports-render and chat-test compilation errors.
Those owners corrected their own files before the successful build.
Log of the stopped attempt: `/tmp/spaceport-qnz9/ydr4-themes-final.log`.

The changed menu-bar consumer scope passed all 16 tests.
Log: `/tmp/spaceport-qnz9/ydr4-themes-menu-consumers.log`.
The complete sidebar width fixture exposed an old retained-field assumption after reopening.
The fixture now retrieves the current field and checks its new identity and empty text.
It preserves the real composition, selection, paste, and layout checks on that current field.
The complete fixture then passed.
Logs: `/tmp/spaceport-qnz9/ydr4-themes-width-final.log` and `ydr4-themes-width-green-final.log`.
The initial `panels` filter selected no standalone tests; it is not counted as verification.

Root independently reviewed capture order, field identity, retained operation ownership, and the actual tests.
That review found no material defects.

## Checks and limits

The owned-file Rust 1.99.0 format check and full diff check passed.
File-size and sorted-region checks passed.
The first localization run stopped on a concurrent Group selector annotation.
After that owner corrected the annotation, localization passed with 2172 messages in 114 files.
Logs: `/tmp/spaceport-qnz9/ydr4-themes-format-final.log`, `ydr4-themes-structure.log`, and `ydr4-themes-i18n-final.log`.
The integration owner retains the final aggregate Clippy and feature-build gates.
The smoke input consumer was adapted to the optional field accessor; native execution remains pending.

All Cargo and artifact runs serialize through `/usr/bin/lockf -k /tmp/spaceport-qnz9/cargo.lock`.
Cargo uses locked dependencies, Rust 1.99.0, and `CARGO_INCREMENTAL=0`.
The shared target is `.worktrees/upstream-fixes/target`.
Builds report only the existing debug-linker unwind and `block 0.1.6` notices in the retained green logs.
Native animation, keyboard/IME delivery, screen-reader behavior, and performance checks remain pending.
This is one completed local popup family; the whole `ydr4` scope remains open.
