# Saved-state native qualification

This test-only follow-up targets the saved-state fix in draft PR165.
Its starting product source is `8a8c332b28dea2d9b4adb541162134f6b177a024`.
It uses neutral fixture code from PR164 without importing that branch's product changes.

## Scenario

The Linux x64 fixture owns one private Xvfb display and one pinned Herdr v0.9.5 server.
It verifies the binary digest before starting that server.
Each normal GUI launch records its process ID, control window, and native X11 window.

Launch A loads synthetic valid and rejected tabs beside a hidden layout reference.
It retains tab 2 and allocates Devices tab 301 above hidden page 300.
After normal quit, it checks the actual saved files and exact private rejected backup.
The daemon must survive the GUI exit.

Launch B reads both files from A without fixture rewriting.
It checks the saved entries through the real CLI and closes only tab 301.
After quit, it records the resulting file bytes and hashes.

Launch C receives a synthetic layout with duplicate page identity fields.
It must retain the readable tab and refuse new Devices tabs.
It captures fresh warning pixels at 900px and 640px before inspecting either width.
Both primary files must remain unchanged after quit.
The daemon must survive every GUI, then exit after the fixture signals its retained child.
The fixture checks its startup trap and reaps its exact children.

The evidence includes source, process/window receipts, state bytes, hashes, backup mode, screenshots, and cleanup.
Available failure evidence uploads even when the test fails.
The completion receipt requires successful recovery, restart, refusal, cleanup, and readable guidance at both widths.

## Verification

The first native attempt ran in CI44917715586, job136886927103, from source `825ec45e4`.
All three GUI phases reached successful exits before the client-backed shutdown witness failed.
Artifact14711539874 retains the state receipts, backup, hashes, and both native capture widths.
Independent byte and hash checks match the recorded files.
Actual PNGs show clipped restart advice at 900px and clipped filenames at 640px.
This failed attempt has no completion receipt.

The witness now reads the bounded private protocol directly and checks the server's peer PID.
It validates repeated shutdown notices and requires their expected reason before clean EOF.
It does not require the separate PR160 client correction.
Independent re-review approves that observer correction.
A headless regression measures the actual repair flash at both widths against a short confirmation.
Source `e9aa0abbf` reaches the intended native clipping assertion after full cleanup.
Its Linux ARM all-features job also fails the expected one-row geometry assertion.
Parent `244a7d529` then replaces truncation with normal wrapping in the existing text container.
Combined source `33e289a6c` passes that geometry case, with 3686 app cases passing on Linux ARM all-features.
Its native PNGs show the complete message, both filenames, and restart advice at both widths.
That native run still fails because OCR reads filename periods as commas.
The scorer now accepts only that evidenced substitution within each complete known filename.
It still requires both filenames and the complete repair and restart clauses.
Pure cases retain the two clipped transcripts and a missing filename as rejected controls.
Raw OCR and PNGs remain unchanged. Independent review approves the recognition correction.
Qualification source `7bb50dcc1` passes complete CI44921341819.
The exact native scenario passes in Linux x64 job136898096754.
The same job passes the geometry case and both OCR control tests.
Its app suite reports 3686 passed, zero failed, and nine ignored.
The [receipt](saved-state-recovery/native/green.json) records the source, artifact digest, state hashes, exact children, and cleanup.
Root inspected both final PNGs and verified the downloaded state and backup bytes.
Later evidence changes record those results without changing the qualified product or test code.

- [x] Independent task review approved specification compliance and code quality.
- [x] `just format-check`, `just check-file-size`, and `python3 scripts/check-sorted.py` pass.
- [x] `actionlint .github/workflows/ci.yml` and `bash -n scripts/install-linux-deps.sh` pass.
- [x] `python3 scripts/release/test-release-security.py` passes all three offline workflow checks.
- [x] `git diff --check` passes. The fixture contains synthetic data and owned process handles.
- [x] Pass `saved_state::recovery::native_saved_state_recovery` in disposable Linux x64 CI44921341819.
- [x] Inspect both final screenshots from `7bb50dcc1` for the full warning, filenames, and restart advice.
- [x] Verify the source, state hashes, backup, process exits, and completion receipt from that run.

No local product build, test, compiler, app, daemon, VM, or SSH probe ran.
The warning retains its existing two-second lifetime.
The wrapping correction fixes observed clipping. The scorer correction passes the same native scenario and its clipped-message controls.
Inspect the actual PNGs before accepting native readability; OCR alone does not prove glyph visibility.
This test does not qualify macOS, Windows, audible speech, performance, WSL, or remote SSH.
Those checks retain their existing status.
PR125 stays a draft checkpoint. No main merge or release is authorized.
