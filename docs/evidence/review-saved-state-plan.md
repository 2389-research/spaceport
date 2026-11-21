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

- [ ] Write real-file regression tests against unchanged production code.

```rust
let mut store = Store::load(Some(path.clone()));
let opened = store.open(Scope::endpoint("local"), "w", Some(Location::Devices), None);
assert!(opened.is_none());
assert!(store.writer.is_none());
assert_eq!(std::fs::read(&path).unwrap(), original);
```

- [ ] Cover mixed valid/rejected entries, torn JSON, read failure, backup failure, oversized files, and missing-file controls.
- [ ] Check exact backup bytes, unique names, and Unix mode 0600.
- [ ] Reserve IDs from invalid tabs, saved `next`, and layout picks, tabs, and hidden references.
- [ ] Cover invalid layout entries, maximum IDs, layout-only writes, last-layout removal, and a second restart.
- [ ] Finish real writers before inspecting persisted bytes. Do not poll or sleep.
- [ ] Review the tests independently before committing.
- [ ] Commit and publish the tests-only draft. Record the exact head and expected assertion failures from CI.

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
Expose `Store::restoration_blocked(&self) -> bool` for actual admission callers.
Keep file I/O before GPUI startup and writer completion off the UI thread.

- [ ] Use `tempfile::Builder` in the source directory with a `<filename>.rejected-` prefix.
- [ ] Write every original byte and call `sync_all()` on the backup file.
- [ ] On Unix, sync the parent directory before enabling replacement writes.
- [ ] Retain the unique backup with `keep()`. Leave ordinary writer syncing unchanged.
- [ ] Reuse strict entry validators for independently recoverable entries.
- [ ] Preserve strict `parse` behavior for its existing callers and schema tests.
- [ ] Block allocation and replacement writers when a read, backup, or reference-envelope check fails.
- [ ] Retain readable tabs while blocked.
- [ ] Reserve raw numeric IDs using saturating arithmetic. Keep checked allocation.
- [ ] Add a defaulted `next_tab_id` field beside saved layouts.
- [ ] Preserve that floor when layouts change or the last layout is removed.
- [ ] Test actual file preservation and typed error sources for the new helper.
- [ ] Review the production diff independently before committing.
- [ ] Publish the fix and require the Task 1 assertions to pass in CI.

## Task 3: Refuse Unsafe New Tabs Before Side Effects

**Files:**

- Modify `crates/sheprd-gpui/src/devices_overview.rs`.
- Modify `crates/sheprd-gpui/src/browser/view.rs` if its corresponding test action needs the same message.
- Modify `crates/sheprd-gpui/src/core_terminals.rs`.
- Add focused GPUI topic tests and localized repair guidance.

**Interfaces:** Use the Store restoration flag and the existing error localization boundary.
Check `Purpose::Tab` before connecting or queueing terminal creation.
Keep existing terminal splits available when their identities are already known.

- [ ] Add headless tests for the actual Devices action and Core new-tab path.
- [ ] Verify blocked state emits repair guidance and queues no terminal request.
- [ ] Retain normal creation and existing-tab controls.
- [ ] Prove the tests fail for the expected behavioral reason in disposable CI.
- [ ] Implement the narrow refusal and localized message.
- [ ] Review the actual callers and test evidence independently.
- [ ] Require the new tests and full configured CI gates to pass.

## Task 4: Qualify and Publish the Focused Replacement

**Files:** Update this plan and the replacement PR QA checklist.

- [ ] Inspect every commit and the complete branch diff.
- [ ] Review secrets, destination visibility, changed paths, file sizes, and formatting.
- [ ] Record exact CI heads, job IDs, RED assertions, and GREEN results.
- [ ] Exercise synthetic saved state across restart on a disposable native desktop when available.
- [ ] Capture real repair guidance screenshots if visible behavior changes.
- [ ] Leave unavailable native checks unticked and name their limits.
- [ ] Refresh PR metadata and verify its remote body, base, head, and draft state.
- [ ] Update the existing dashboard row without changing other rows or people's feedback.
- [ ] Leave PR #125 preserved as a draft checkpoint.

## Evidence

This plan records intended work. No replacement RED/GREEN result exists yet.
Historical recovery code is in `150eed475c4097c63dff86edf8e2b215c64a1d02`.
The final historical restart fixture is in `aaff481f9703bffdb138374eeb9047a7b58e900e`.
Those commits contain unrelated changes and must not be cherry-picked whole.
