# Current phase handoff

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
  hosted rerun remains pending. The protected signing environment
  exists with main-branch and `v*` tag rules, the Apple API secrets and the
  approved identity variable. Only `harperreed` has repository write access.
  Certificate export is waiting for macOS keychain authorization; an approved
  matching P12/password is still needed before signing. Native verification
  of the final-image checker passed on the existing signed debug DMG; optimized
  hosted signing/publication and full native GUI QA remain unproved.
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
