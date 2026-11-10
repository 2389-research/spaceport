# Current phase handoff

- About shipping: `68c5e6c6d8d409be03ab1491bd655cdf84b74012` is committed,
  fast-forwarded to main and pushed with `feat/about-attribution`.
  [Full CI 42906153627](https://github.com/harperreed/spaceport/actions/runs/42906153627)
  exposed one missed test: the worktree PR banner uses the shared repository
  constant, but its expected URL still named upstream. Linux ARM, macOS and
  Windows logs confirm that sole app test failure. It reproduced locally.
  The assertion now requires the correct Spaceport PR URL. Full local app
  all-feature unit tests passed 3,840 cases with nine existing opt-in ignores.
  Correction `489f8c205dea3807d0d044fa4078b1586cd3bfcd` is committed, merged
  and pushed. [Exact-source CI 42907726241](https://github.com/harperreed/spaceport/actions/runs/42907726241)
  passed every required job; the source gate independently confirmed it.
  The superseded red run was explicitly cancelled to free its queue.
  Immutable tag `v20261110.5` now selects that qualified source.
  [Preview workflow 42909921543](https://github.com/harperreed/spaceport/actions/runs/42909921543)
  passed all five jobs and [published the prerelease](https://github.com/harperreed/spaceport/releases/tag/v20261110.5).
  Both Apple submissions were Accepted. The downloaded public DMG matches its
  checksum and the retained signed artifact byte-for-byte. Local signatures,
  stapled tickets, Gatekeeper, identity and CLI checks passed. Bundled NOTICE,
  attribution messages and standard nebula resources were checked too.
  [Exact qualification](../evidence/t12-macos-preview-20261110.5-signed.md)
  records source, digests, UUIDs and limits. No personal app/runtime changed.
  Next preview: use a fresh unused version on fully qualified main source.
  Do not alter `.3`, `.4` or `.5`, reuse their versions, or retry submissions.

- About attribution follow-up: Doctor Biz approved Spaceport by 2389 Research,
  Inc., separate modifications/original Sheprd copyright rows, both upstream
  team credits, Apache-2.0 and third-party notices. Implemented
  on `feat/about-attribution` in `.worktrees/about-attribution`, based on
  `7143eb821`. Spaceport's repository is primary; upstream links stay under
  credits. NOTICE prepends the new attribution and retains the prior text.
  Eight About tests passed in default/all-feature builds, including a 360px
  layout and link/close behavior; the real native profile/About check passed.
  Ten packaging/notice tests, strict all-target/all-feature app Clippy,
  formatting, i18n, source bounds, sorting and diff checks passed. Native paint
  inspection now skips FSI/PDI ink bounds without removing the source markers
  or weakening visible text comparisons. Review found no blockers.
  Broader `just test-sidebar` passed glyph/bidi/About checks but failed its
  later macOS decoy activation (`key=false`, `app_active=false`). The unchanged
  `7143eb821` baseline failed identically; do not claim the broader suite passed.
  Logs are retained under this worktree's ignored `target/about-attribution/`.
  Doctor Biz explicitly authorized merge to main, push and a fresh tag.
  This follow-up shipped as `.5` above. Retain the independent native host
  activation limitation and distinguish source-native checks from full UI
  qualification of the final signed image.

- Live tag activation: all five signing/notary secrets and the identity variable
  are configured in `macos-preview`. Doctor Biz supplied the primary 2389
  credential directory; its P12/password verified the exact approved Developer
  ID certificate. The unused interactive keychain export was withdrawn and its
  private helper/password removed; no local access policy changed. No credential
  question remains. Only `harperreed` has repository write access.
- Earlier `.4` qualified release source was `f490516d7`.
  [Full exact-source CI passed](https://github.com/harperreed/spaceport/actions/runs/42890348802),
  including every required platform/feature matrix, both Windows native and
  performance jobs and both optimized Linux executable checks. `ec5b5795a`
  keeps Core backends alive while test subscriptions read their events; the
  formerly failing Windows combinations now pass. The preceding `abefc1885`
  run also passed the real Windows installer cycle. Local focused tests,
  strict Core Clippy, formatting, size and sorting checks pass.
  Preview notes now use the shared git-cliff generator, bounded by the last
  published ancestor or recorded fork baseline; failed tags do not split them.
  Fresh immutable tag `v20261110.4` now points to that green source.
  [Preview workflow 42893227129](https://github.com/harperreed/spaceport/actions/runs/42893227129)
  passed all five jobs and [published the prerelease](https://github.com/harperreed/spaceport/releases/tag/v20261110.4).
  Both Apple submissions were Accepted. The public DMG matches the retained CI
  artifact byte-for-byte and passed local signatures, tickets, Gatekeeper,
  compiled identity and CLI checks. Actual icon resources match the approved
  standard nebula assets. [Exact qualification](../evidence/t12-macos-preview-20261110.4-signed.md)
  records source, hashes and UUIDs. Do not repeat either successful submission.
  Next: use a fresh unused tag on another fully qualified main commit for a
  later preview. Broader runtime/update/native app acceptance remains open.
- First tag `v20261110.3` is immutable at `f81bbe64d`.
  [Its workflow](https://github.com/harperreed/spaceport/actions/runs/42876401991)
  validated the tag and built/smoke-tested the optimized arm64 unsigned app.
  Signing has not started: its exact-commit CI has a Windows helper lint failure.
  Do not move/delete/rerun that tag or submit its artifact. The new `.4` run
  above is a separate build on qualified source; monitor its own receipts.
  No notarization submission or GitHub Release occurred for the failed `.3` run.
  Earlier historical notes below retain their narrower phase scope.

- Branding follow-up: D63 selects the nebula rocket. See
  [artwork evidence](../evidence/t12-nebula-artwork.md) and
  `wip/spaceport-rocket-branding` for source/export checks. The approved PNG is
  `design/spaceport-icon/rocket-nebula.png`. All platform artwork and social
  cards derive from it. [Signed preview 20261110.2](../evidence/t12-macos-preview-20261110.2-signed.md)
  is ready under the integration checkout's `target/test-builds/signed/20261110.2/`.
  It is a Developer ID signed, notarized and stapled arm64 debug DMG. Both
  Apple submissions were Accepted; signatures, Gatekeeper, compiled identity,
  shipped CLI and nebula resource checks passed. Earlier access failures
  cleared without changing keychain policy or importing a P12. Do not repeat
  its submissions. The earlier unsigned ZIP remains separately preserved.
- Native Dock/About/full GUI verification remains unproved. The exact private
  test processes are stopped; no personal runtime or installed app was changed.
  D64's [tag-triggered workflow](../evidence/t12-tagged-macos-workflow.md) is
  implemented and locally verified on `feat/tagged-macos-release` in
  `.worktrees/tagged-macos-release`, based on `a1e594d31`. Doctor Biz then
  explicitly authorized push and activation. Commits `b35fb5040` and `0947e768d`
  are on GitHub main, which now contains the current Spaceport development
  history. [First hosted CI](https://github.com/harperreed/spaceport/actions/runs/42871509640)
  exposed release-test portability, inherited Rust lint errors and obsolete
  native fixtures. Fixes `cb359eeb0`, `dbe42773b` and `c6a84479b` passed local
  release checks, strict Clippy and the isolated native profile test. Their
  hosted reruns progressed to the qualified source recorded above. The protected
  signing environment exists with main-branch and `v*` tag rules, the Apple API secrets and the
  approved identity variable. Only `harperreed` has repository write access.
  The earlier certificate export blocker is resolved by the supplied matching
  P12/password as recorded above. Native verification
  of the final-image checker passed on the existing signed debug DMG; optimized
  hosted signing/publication has now passed as recorded above; full native GUI
  QA remains unproved.
  The older phase handoff below retains its separate runtime/ownership scope.

- Task/requirements: bounded T06 SP-009/SP-003 chat lifecycle policy removal;
  T10 exclusion evidence and T12 refreshed signed preview. Task/acceptance/gap
  statuses remain in plan JSON, not duplicated here.
- Integration: `wip/spaceport-t00`, application source `9bbc81c70889f1f6da29ed2cecee493e8c3afe35`;
  source commits `2041aa09c` and `e4303a6b0`. Main was not merged or pushed.
- [Lifecycle evidence](../evidence/t06-chat-lifecycle-policy-removal.md) records
  removed/retained boundaries, tests and inherited strict-lint failures.
  [Process evidence](../evidence/t06-adapter-detach-process.md) separates
  observed unit zero-client state from process drop/reconnect continuity.
- [Preview 20261110.1](../evidence/t12-macos-preview-20261110.1.md) contains this
  source, exact signed artifact identities and successful private fixtures on
  the shipped signed helper. Delivery remains in the integration checkout's
  `target/test-builds/signed/20261110.1/`; previous preview is retained.
- Core fixture: explicitly installed 0.1.19, source revision unknown; executable
  hash is in evidence. Pinned 0.1.18 source remains a distinct read-only reference.
- [Duplex route audit](../evidence/t06-core-duplex-route-audit.md) and retained
  schema narrow the external G-CHAT dependency; no RPC or second transport was
  invented. Remote rich chat and final native/real-agent parity are unproved.
- Core schema CLI unexpectedly autostarted a private daemon. Its exact owned
  process was identified and cleaned; no unknown side effect remains from that
  probe. Use direct-serve Sandbox for further runtime inspection.
- The older lifecycle phase did not authorize installation, main merge, push,
  publication or hosted credential setup. D64's later activation instruction
  now authorizes the GitHub work above, while personal runtime tests and
  upstream source edits remain outside scope.
  No product question is waiting. PACT writes remain paused on the separate
  recorded reference incident; its untracked object is preserved.
- Next single executable step: from the package root run
  `.venv/bin/python tools/spec.py context T06`. Continue the remaining
  session/config ownership audit within the existing scope; do not re-add
  conversion/recovery to hide a missing backend route. An actual final-preview
  native test needs an explicitly isolated fixture and an unlocked desktop.
- This code phase has landed in development integration. Retire this session
  after evidence commit; the next phase starts from these records. Task
  worktrees belong under the original checkout's `.worktrees/`.
