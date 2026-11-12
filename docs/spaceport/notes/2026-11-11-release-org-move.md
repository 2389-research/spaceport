# Release recovery and repository move

Doctor Biz requested moving Spaceport to `2389-research`, pausing Windows
builds until its performance failure is fixed, and delivering a new release.

- [x] Inspect failed main CI and queued preview; Windows performance is the
  latest main failure. Preview 43098075293 has no jobs or signing side effects.
- [x] Transfer the existing repository, preserving history, PRs and secrets.
- [x] Update active repository links, workflow eligibility and signed-feed URLs.
- [x] Pause Spaceport Windows check/native/package jobs; require their skipped
  state without weakening macOS/Linux checks or changing performance budgets.
- [ ] Test policy and release contracts, review, commit and push a focused PR.
- [ ] Merge after CI, qualify the exact main commit and reserve a fresh tag.
- [ ] Verify signing, notarization, published DMG and signed feed, or record
  the exact external blocker and retained diagnostic evidence.

Use `.worktrees/release-org-move` on `fix/release-org-move`. Keep previous tags
immutable. Keep signing keys, Team ID, bundle identity and the Sparkle public
key unchanged. Historical evidence keeps the repository URLs used at the time.
Windows performance remains tracked by Kata `h8g5`; pausing builds is not a fix.

The old queued preview also exposed a release-history failure: its uninitialized
run title was a commit subject. History now reserves its version from a valid
push tag and full source SHA; malformed/manual metadata still refuses. Twelve
history tests pass after reproducing the failure. Initial live API validation
hit a TLS handshake timeout; no remote state changed.

Local validation: four transition tests, nine CI policy tests, fifteen release
gate tests, 24 release security tests (one opt-in SBOM skip), 62 preview contracts
(five optional native/SDK skips), and eighteen Rust report/About/banner tests
passed. Format, file-size, sorted and i18n checks pass. Spec render/check pass.
Review found no remaining issues. Full platform lint/test/build verification
and signed artifact delivery remain pending CI. Native click/glyph QA was not
run because the desktop is locked.
