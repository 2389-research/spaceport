# T12: tag-triggered macOS preview workflow

D64 extends the existing preview workflow on top of development integration
`a1e594d31`. Implementation is in the isolated `feat/tagged-macos-release`
worktree; it has not been committed, merged, pushed or run on GitHub.

An owner push of `vYYYYMMDD.COUNTER` selects an optimized arm64 preview. The
request validator checks the event, version, exact checkout, current tag target
and main ancestry. A separate gate requires complete successful main CI for
that SHA. The existing CI workflow now permits Spaceport with its own owner
mapping; upstream CI rules remain intact. Unsigned building can overlap CI,
but signing waits for both.

The protected `macos-preview` job imports the matching Developer ID identity,
signs, notarizes and staples through the existing signer. It checks the final
DMG's metadata and digest, signatures, tickets and Gatekeeper, then mounts it
read-only and runs identity and windowless CLI checks against the mounted app.
Only after those checks does it write `SHA256SUMS`. The tag-only publication job
rechecks live refs, downloads only this run's artifact and verifies its checksum
before creating a GitHub prerelease containing the DMG and checksum. It alone
has `contents: write`; signing secrets appear only on the signing step.
Manual main runs retain artifact-only behavior.

Review found a history denial: an invalid `v*` tag would create an unknown title
and block all later reservations. History now recognizes that exact title shape
and skips invalid version tokens, which validation and signing cannot accept.
Unknown title shapes, malformed records, duplicate IDs, reruns and previous
reservations of valid versions still fail closed. Regression tests cover this.

Verified locally on 2026-11-10:

- TDD reproduced missing tag validation and workflow support before changes.
  Real isolated Git fixtures exercise annotated/lightweight tags, unmerged
  commits, moved tags, invalid requests and manual exact-SHA selection.
- `just release-check`: passed; 294 release tests with eight existing skips,
  24 security tests with one opt-in skip, Bash syntax, all-workflow actionlint,
  zizmor and diff whitespace checks. The first run exposed the existing policy
  test interpreter's missing `!=` support; it now evaluates that operator and
  tests repository/event/ref/actor combinations for Spaceport and upstream.
- `just check-file-size check-sorted`: passed.
- Package `tools/spec.py check`, using the integration checkout's existing
  `.venv/bin/python`: passed, including schema and generated-view checks.
- `verify-macos-preview.sh 20261110.2 debug` passed against a private copy of the
  already signed DMG, SHA-256
  `d64535c27f2bf1751bef2436bcc4bf99b5f8b119124a8045f519d123ef38a536`.
  This used real Apple tools, mounted the real app, ran its actual CLI and
  detached the exact private mount. Changing its metadata to a wrong Team
  correctly failed before producing a checksum. No signing submissions occurred.
- Independent review and final fresh-eyes review completed. Final review added
  clearer verification failures, signal cleanup and refusal to overwrite a
  checksum. The final native verifier rerun passed after those changes.

Local full logs are under this worktree's ignored
`target/tagged-release-verification/`. The release suite includes controlled
Apple tool fixtures; those are control-flow tests, not native signing evidence.
The native run above used the existing **debug** preview, not a new optimized
hosted build. No native GUI, Windows/Linux build, full Rust workspace test or
hosted publication was performed for this workflow change.

Activation remains blocked on merging the workflows to main, full CI passing
there, and provisioning the protected environment with its branch/tag rules,
2389 P12/export password and Apple API secrets. The environment was absent in
the read-only GitHub check; no credentials or repository settings were changed.
The local installed identity cannot supply an unknown P12 export password to
GitHub. Fix these before pushing a release tag. Broader T12/G-RELEASE and native
GUI gates remain open; this does not enable the inherited production pipeline,
an updater or other-platform publishing.

Known low-severity tool notices: local macOS 26 warns that `hdiutil` is
deprecated; the workflow targets macOS 15, where those commands remain its
supported path. The all-workflow zizmor run reports its existing offline mode
and unsupported `python3` shell analysis notices, with no findings. Neither
notice was hidden or counted as native GUI proof.
