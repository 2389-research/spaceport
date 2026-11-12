# Release recovery and repository move

Doctor Biz requested moving Spaceport to `2389-research`, pausing Windows
builds until its performance failure is fixed, and delivering a new release.

- [x] Inspect failed main CI and queued preview; Windows performance is the
  latest main failure. Preview 43098075293 has no jobs or signing side effects.
- [x] Transfer the existing repository, preserving history, PRs and secrets.
- [x] Update active repository links, workflow eligibility and signed-feed URLs.
- [x] Pause Spaceport Windows check/native/package jobs; require their skipped
  state without weakening macOS/Linux checks or changing performance budgets.
- [x] Test policy and release contracts, review, commit and push a focused PR.
- [x] Merge after CI, qualify the exact main commit and reserve a fresh tag.
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

PR #10 merged after full successful CI 43150221075. Source commit
`91b587818f229a278c2c678c23425db4ea77e0e8`; main merge
`e874b260c9d71f75cb5f3f30418546f810b10417`. Exact-main qualification
is running as CI 43151863635. All three Windows jobs skipped as requested;
macOS/Linux checks and both optimized macOS CLI suites passed in PR CI.
Candidate version `20261112.1` uses the UTC date and was unused when inspected.
CLI REST quota exhaustion lasts until 04:06:15 UTC; connector reads remain
available. Local git-cliff is absent; the release workflow installs the pinned
version and generates the authoritative notes. No preview tag has been pushed.

Exact main CI 43151863635 passed. The actual check-ci.py gate accepted
`e874b260c9d71f75cb5f3f30418546f810b10417`; live preview history validation
also passed after the CLI quota reset. Pinned git-cliff 2.12.0 was installed
under the ignored task tools directory; generated notes for this exact merge
commit were reviewed against published ancestor v20261110.5.

Annotated tag `v20261112.1` now points to the qualified merge commit. The
new macOS preview workflow 43153449527 is running with the correct rendered
title, rather than stuck before job creation. Signing, notarization and
public artifact/feed verification remain pending.
