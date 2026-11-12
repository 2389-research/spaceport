# Release recovery and repository move

Doctor Biz requested moving Spaceport to `2389-research`, pausing Windows
builds until its performance failure is fixed, and delivering a new release.
D66 records the approved scope; Windows performance remains Kata `h8g5`.

- [x] Transfer the existing repository, preserving history, PRs and secrets.
- [x] Update active repository links, workflow eligibility and signed-feed URLs.
- [x] Pause Spaceport Windows check/native/package jobs and require their
  skipped state without weakening macOS/Linux checks or changing budgets.
- [x] Fix queued-run version reservations using valid push-tag/source metadata.
- [x] Run focused tests, review, merge PR #10 and qualify exact main CI.
- [x] Review generated notes and push fresh annotated tag `v20261112.1`.
- [x] Verify signing, notarization, public DMG and signed feed.

Repository: https://github.com/2389-research/spaceport (ID 1449575010).
The transfer preserved all six signing secret names, the identity variable,
and main/v* environment policies. Organization members now inherit repository
permissions. Trusted release actor, Team, bundle identity and Sparkle key stay
unchanged. Historical evidence retains the URLs used at the time.

PR #10 source is `91b587818f229a278c2c678c23425db4ea77e0e8`.
Qualified main/tag source is `e874b260c9d71f75cb5f3f30418546f810b10417`.
PR CI [43150221075](https://github.com/2389-research/spaceport/actions/runs/43150221075)
and exact-main CI [43151863635](https://github.com/2389-research/spaceport/actions/runs/43151863635)
passed. The real `check-ci.py` gate accepted the exact main SHA. All three
Windows jobs skipped; Windows acceptance is not claimed.

Preview [43153449527](https://github.com/2389-research/spaceport/actions/runs/43153449527)
passed all five jobs and published the signed arm64 preview.
The public DMG passed checksum, signature, ticket, Gatekeeper and CLI checks.
The signed feed and archive verified. The old feed URL returned identical bytes
after redirecting. The mounted bundle passed Sparkle policy validation.
See the [qualification record](../evidence/t12-macos-preview-20261112.1-signed.md).
Old run 43098075293 remains queued with no jobs; GitHub refused cancellation.
Its version stays reserved. No old tag was moved and no submission was retried.

Local transition, release-policy and preview/history contracts passed after
observed red/green regressions. Eighteen Rust report/About/banner tests passed.
Format, file-size, sorted, i18n, specification render/check and diff checks pass.
Pinned git-cliff 2.12.0 generated the reviewed notes for the exact merge source
from published ancestor v20261110.5. Live GitHub history validation passed after
the CLI REST quota reset. Optional SDK/native/SBOM skips are listed in PR QA.
The build Mac remains locked; native click/glyph and installed-update QA were
not run during this recovery.

Task worktree: `.worktrees/release-org-move`; documentation branch
`docs/release-org-move-qualification`. The requested recovery is complete.
Next: test Check for Updates from the installed baseline on an unlocked Mac.
Keep Windows builds paused until native performance qualification passes.
