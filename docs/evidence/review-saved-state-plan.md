# Saved State Preservation Implementation Plan

> **For agentic workers:** Use subagent-driven-development for each scoped task. Root owns integration and Git publication.

**Goal:** Preserve rejected saved files and prevent new tabs from reusing saved layout identities.

**Architecture:** Keep persistence in `sheprd-browser` and file operations in `sheprd-state-file`.
Preserve rejected bytes before allowing replacement writes.
Persist the layout ID floor beside layouts in the same atomic file.
Refuse new tab creation when restoration cannot establish a safe ID floor.

**Tech Stack:** Rust, serde_json, tempfile, existing GPUI tests, disposable GitHub CI.

## Global Constraints

- Base this branch on `234dfdd1b852b76d6bf2100ef8c8f820ba6e5248`, the qualified Windows fixture stack.
- Preserve `Store::load(Option<PathBuf>)` and `Layouts::load(Option<PathBuf>)`.
- Preserve strict parsing, retired payload validation, 256-entry limits, and 1 MiB read limits.
- Keep backups private, unique, exact, and unpruned.
- Never read personal session state or use a personal daemon.
- Run product verification only in disposable GitHub CI.
- Local work permits Git, Rust formatting, source review, and static validators.
- Do not run local builds, tests, compilers, product lints, apps, daemons, VMs, or SSH probes.
- Keep Mac foreground, speech, and performance checks pending.
- Do not merge main, release, rewrite history, discard work, or edit Herdr.
- Keep each Rust file below 1,000 lines.
- Keep broader Core capacity reservations and request lifecycle changes outside this slice.

## Task 1: Prove File Loss and Identity Reuse

**Files:**

- Add `crates/sheprd-browser/src/store/tests/recovery.rs`.
- Add `crates/sheprd-browser/src/layouts/tests/recovery.rs`.
- Register those files in their existing parent test modules.
- Add `--no-fail-fast` to the four workspace test commands in `.github/workflows/ci.yml`.

**Interfaces:** Use existing loaders, `Store::open`, `Store::get`, `Layouts::save`, and private writer completion in unit tests.
Do not add production methods or missing-symbol assertions for the RED checkpoint.

- [x] Write real-file regression tests against unchanged production code.

```rust
let mut store = Store::load(Some(path.clone()));
let opened = store.open(Scope::endpoint("local"), "w", Some(Location::Devices), None);
assert!(opened.is_none());
assert!(store.writer.is_none());
assert_eq!(std::fs::read(&path).unwrap(), original);
```

- [x] Add cases for mixed entries, torn JSON, read failure, backup failure, oversized files, and missing-file controls.
- [x] Add assertions for exact backup bytes, unique names, and Unix mode 0600.
- [x] Add ID cases for invalid tabs, saved `next`, and layout picks, tabs, and hidden references.
- [x] Cover invalid layout entries, maximum IDs, layout-only writes, last-layout removal, and a second restart.
- [x] Finish real storage writers before assertions. Do not poll or sleep.
- [x] Review the tests independently before committing.
- [x] Publish the tests-only draft and inspect actual assertion failures on its exact source.

CI uses existing default and all-feature workspace jobs on Linux, macOS, and Windows.
Compile failures do not prove these regressions.
Cargo's `--no-fail-fast` runs later test executables after a failure and retains the failing exit status.
This lets the same baseline run report browser and GPUI regressions.
[Cargo test options](https://doc.rust-lang.org/cargo/commands/cargo-test.html#test-options).

## Task 2: Preserve Rejected Bytes and Recover Valid Entries

**Files:**

- Modify `crates/sheprd-state-file/src/lib.rs` and its topic tests.
- Modify `crates/sheprd-browser/src/store.rs` and `store/saved.rs`.
- Modify `crates/sheprd-browser/src/layouts.rs` and `groups.rs`.
- Modify typed errors only where a new failure contract needs one.

**Interfaces:** Add `sheprd_state_file::preserve_rejected(&Path, &[u8]) -> Result<PathBuf>`.
Expose `Store::check_new_tab_id(&self) -> Result<()>` for actual admission callers.
Keep its private checked increment shared with `restore_tab`.
Distinguish `RestorationBlocked` from `TabIdsExhausted` through typed browser errors.
Keep valid existing-tab saves enabled when IDs are exhausted.
Keep file I/O before GPUI startup and writer completion off the UI thread.

- [x] Use `tempfile::Builder` in the source directory with a `<filename>.rejected-` prefix.
- [x] Write every original byte and call `sync_all()` on the backup file.
- [x] On Unix, sync the parent directory before enabling replacement writes.
- [x] Retain the unique backup with `keep()`. Leave ordinary writer syncing unchanged.
- [x] Reuse strict entry validators for independently recoverable entries.
- [x] Preserve strict `parse` behavior for its existing callers and schema tests.
- [x] Block allocation and replacement writers when a read, backup, or reference-envelope check fails.
- [x] Retain readable tabs while blocked.
- [x] Reserve raw numeric IDs using saturating arithmetic. Keep checked allocation.
- [x] Inspect identity fields in the original bytes before a JSON map can discard duplicate keys.
- [x] If identities are ambiguous, retain readable entries and block allocation and replacement writes.
- [x] Keep unrelated invalid scalar entries recoverable without making valid siblings read-only.
- [x] Add a defaulted `next_tab_id` field beside saved layouts.
- [x] Preserve that floor when layouts change or the last layout is removed.
- [x] Test real-file preservation and typed error sources for the new helper in disposable CI.
- [x] Review the production diff independently before committing.
- [x] Publish the fix and require the Task 1 assertions to pass in CI.

## Task 3: Refuse Unsafe New Tabs Before Side Effects

**Files:**

- Modify `crates/sheprd-gpui/src/devices_overview.rs`.
- Modify `crates/sheprd-gpui/src/browser/view.rs` if its corresponding test action needs the same message.
- Modify `crates/sheprd-gpui/src/core_terminals.rs`.
- Share the refusal message through `browser.rs` and `browser/view.rs`.
- Use it in `browser/closed.rs`, `core_terminals/commands.rs`, and `core_terminals/workspace_ui.rs`.
- Add focused GPUI topic tests and localized repair guidance.

**Interfaces:** Use the Store restoration flag and the existing error localization boundary.
Check `Purpose::Tab` before connecting or queueing terminal creation.
Keep existing terminal splits available when their identities are already known.

- [x] Add headless tests for the actual Devices action and Core new-tab path.
- [x] Verify blocked state emits repair guidance and queues no terminal request.
- [x] Retain normal creation and existing-tab controls.
- [x] Prove the tests fail for the expected behavioral reason in disposable CI.
- [x] Implement the narrow refusal and localized message.
- [x] Review the actual callers and test evidence independently.
- [x] Require the new tests and full configured CI gates to pass.

The maximum layout ID introduces another admission refusal before terminal creation.
A private Core case now checks that boundary without escalating a pending join.
Publish that case in a tests-only follow-up and require its own assertion RED before changing the Core guard.
Preserve ordinary saves for valid existing tabs when their ID counter is exhausted.
This adds no capacity reservation or queued-request lifecycle policy.

## Task 4: Qualify and Publish the Focused Replacement

**Files:** Update this plan and the replacement PR QA checklist.

- [x] Inspect every commit and the complete branch diff.
- [x] Review secrets, destination visibility, changed paths, file sizes, and formatting.
- [x] Record exact CI heads, job IDs, RED assertions, and GREEN results.
- [x] Exercise synthetic saved state across restart on a disposable native desktop when available.
- [x] Capture and inspect real repair guidance screenshots at 900px and 640px.
- [x] Leave unavailable native checks unticked and name their limits.
- [x] Refresh PR metadata and verify its remote body, base, head, and draft state.
- [x] Update the existing dashboard row without changing other rows or people's feedback.
- [x] Leave PR #125 preserved as a draft checkpoint.

The native Linux case uses one private Xvfb desktop and one pinned, owned Herdr server.
Seed synthetic files under the sandbox's `state/spaceport` directory before normal GUI startup.
Launch A must retain tab 2, discard the invalid sibling, and allocate above hidden layout ID 300.
After quitting A, launch B against its untouched files and verify the actual persisted entries.
After quitting B, launch C with an ambiguous layout and verify refusal without replacing either primary file.
Reap each exact GUI child before reading or changing its state files.
Keep the server alive across all three launches and verify that the startup trap stays empty.
Capture fresh warning pixels at 900px and 640px, with source, PID, XID, file hashes, and cleanup receipts.
The existing flash lasts two seconds and truncates text; inspect actual readability before accepting the warning.
The native case must reuse neutral test helpers from PR164 without importing its unrelated product changes.

## Evidence

Implementation `736d86d72` preserves rejected files, identity floors, and Core admission through the reviewed source changes.
Its first CI run `44915649055` stops at the same lint error on Linux, macOS, and Windows.
Importing `Path` makes the existing `Scope::local` qualification redundant.
Use the imported type without changing the function contract.
The next run `44916441550` reaches the inherited schema test and exposes its matching redundant qualification.
Use the imported type there too. The affected storage modules have no remaining equivalent qualification.
Both qualification corrections are committed in `8a8c332b28dea2d9b4adb541162134f6b177a024`.
Its complete configured CI `44916693681` passes.
The [GREEN receipt](saved-state-recovery/green.json) records all job outcomes and five inspected platform logs.
Linux x64, Linux ARM64, and macOS each pass all 41 relevant cases.
Windows x64 and ARM64 each pass 37 cases, omitting four Unix-only cases.
All original and additional baseline regressions and controls pass where compiled.
Native restart behavior and actual warning readability remain separate pending gates in draft PR166.

The first native attempt at `825ec45e4` runs all three GUI phases in CI44917715586, job136886927103.
Its [partial receipt](saved-state-recovery/native/baseline.json) records exact bytes, backups, hashes, and GUI exits.
The copied client-backed witness then fails because its shutdown event requires the separate PR160 correction.
The revised test observes the bounded real protocol directly and keeps the typed shutdown-before-EOF assertion.
It validates Herdr's repeated shutdown reason rather than rejecting the second legitimate notice.
That observer and a headless geometry regression are independently reviewed and published as `e9aa0abbf` in PR166.
CI44919201790, job136891435585, reaches the intended unreadable-warning assertion at both widths.
Its [clipping RED receipt](saved-state-recovery/native/clipping-red.json) records the typed shutdown, EOF, and cleanup receipts.
The renderer correction replaces truncation with constrained, normally wrapping text.
The same native test and actual after images must pass before accepting the correction.
The actual [900px](saved-state-recovery/native/repair-before-900.png) and [640px](saved-state-recovery/native/repair-before-640.png) frames confirm clipped repair guidance.
No complete native qualification is claimed from that failed attempt.

Wrapped product source `244a7d529` passes complete configured CI44920108697.
Combined source `33e289a6c` passes the actual headless geometry case on Linux ARM all-features.
Its PNGs contain the full warning, but OCR reads filename periods as commas.
The narrow test correction accepts only that evidenced substitution within both complete known filenames.
It retains the complete clauses, raw images, raw OCR, and rejected clipped/missing-name controls.
Independent review approves that correction, published as qualification source `7bb50dcc1`.
CI44921341819, job136898096754, passes the native scenario and the Linux all-features workspace suite.
All three focused geometry/OCR tests pass; the app reports 3686 passed, zero failed, and nine ignored.
The [native GREEN receipt](saved-state-recovery/native/green.json) records actual completion, exact children, hashes, backup, and cleanup.
Root inspected the actual [900px](saved-state-recovery/native/repair-after-900.png) and [640px](saved-state-recovery/native/repair-after-640.png) after images.
Both filenames and the full restart instruction remain visible inside the terminal.
Complete qualification CI44921341819 also passes at `7bb50dcc1`.
Mac foreground, actual speech, and performance retain their explicit holds.

### Historical regression baselines

Tests-only head `0cef725631188ec430e9f3501a6a7a79bf2cbab5` has actual behavioral RED in CI44908322454 attempt 2.
Linux x64 all-features job136860276395 reproduces 19 storage and three UI failures.
Windows x64 all-features job136860276365 reproduces 16 storage and two UI failures.
Linux ARM64 default job136860276381 reproduces the same 19 storage and three UI failures.
macOS all-features job136860276384 reproduces the same 19 storage and three UI failures.
Windows ARM64 all-features job136860276352 reproduces the same 16 storage and two UI failures.
The two missing-file controls and existing Devices tab control pass in all five inspected jobs.
The [baseline receipt](saved-state-recovery/baseline.json) records source, checkout, jobs, assertions, controls, and limits.
No product fix or GREEN result is claimed at this checkpoint.

Independent review found that the proposed `Value` fallback loses earlier duplicate identity fields.
The proposed production patch remains uncommitted while identity handling is corrected.
Those cases cover tab IDs, saved counters, tab lists, and layout IDs, picks, and kinds.
They also protect writable recovery of valid siblings beside unrelated invalid scalar entries.
The follow-up contains tests, the plan, and baseline evidence; it contains no production fix.

Follow-up head `38b80ff00a94f720db9e1a66028969268e9fe683` reaches the nine additional assertions in CI44912316144.
Linux x64 all-features job136870656024 has 27 storage and four UI failures, all expected regression cases.
All eight identity cases and the exhausted-ID Core case fail at their intended assertions.
The three original controls still pass.
The [identity baseline receipt](saved-state-recovery/identity-baseline.json) records checkout, cases, assertions, and limits.
The storage ambiguity repair and UI admission changes are now authorized to proceed through review and CI.

Attempt 1 failed checkout after two github.com:443 connection failures.
Root retried the terminal checkout job through GitHub's supported job rerun action.
Attempt 2 passed checkout and the existing workflow gates before running the regressions.
Historical recovery code is in `150eed475c4097c63dff86edf8e2b215c64a1d02`.
The final historical restart fixture is in `aaff481f9703bffdb138374eeb9047a7b58e900e`.
Those commits contain unrelated changes and must not be cherry-picked whole.
