# Published optimized macOS preview 20261110.4

Doctor Biz authorized the tag-triggered signed distribution workflow and its
activation. The complete workflow passed on 2026-11-10 and published the
[macOS prerelease](https://github.com/harperreed/spaceport/releases/tag/v20261110.4).

| Observation | Verified result |
| --- | --- |
| Immutable tag | `v20261110.4` |
| Source | `f490516d7bbe3c213b12e83a2efb4aad836bb0f4` |
| Full main CI | [42890348802](https://github.com/harperreed/spaceport/actions/runs/42890348802), success |
| Preview workflow | [42893227129](https://github.com/harperreed/spaceport/actions/runs/42893227129), all five jobs success |
| App identity / Team | `ai.2389.spaceport` / `HD9NM9NSMK` |
| Version / profile / architecture | `20261110.4` / release / arm64 |
| App notarization | `62a0990f-9430-4645-8d3b-8016101b7d85`, Accepted |
| DMG notarization | `a0e73254-3f7e-4a5a-8a96-18f69a7b16e7`, Accepted |
| DMG | `Spaceport-20261110.4-arm64-apple-darwin.dmg` |
| Size | 32,214,127 bytes |
| SHA-256 | `b2dd0d4f3dcc5e344123a47da008c67fa837b340ece948f0f9218ec13f53d330` |

The tag points to the exact source with complete successful main CI. Its
separate preview CI gate independently verified every required job before
signing. The unsigned build used no signing credentials and smoke-tested the
actual optimized executable. The protected environment supplied the approved
identity and Apple API inputs only to the signer. Both Apple submissions were
Accepted. Their retained UUIDs were inspected; no submission was retried.

The publication job downloaded only the same-run signed artifact, verified its
checksum, generated notes through the shared pinned git-cliff, rechecked live
tag/main ancestry, and created the prerelease with the DMG and `SHA256SUMS`.
All five workflow jobs completed successfully.

## Verification of the published download

The public release's DMG and checksum were downloaded after publication.
`shasum -a 256 -c SHA256SUMS` passed, and `cmp` proved the public DMG identical
to the retained signed workflow artifact. The downloaded artifact ZIP also
matched GitHub's recorded SHA-256:
`aa0c1203f2abf0a86d595eb414bb5ae82af0932a676eebefd1541ef4b98ee0c5`.

`verify-macos-preview.sh 20261110.4 release` passed locally on a copy of the
publicly downloaded DMG with its same-run signing metadata. It verified:

- Strict DMG and deep app signatures, exact app identifier and Team.
- Stapled app/DMG tickets and Gatekeeper acceptance with
  `source=Notarized Developer ID`.
- Compiled version/profile/architecture identity, the mounted executable's
  canonical CLI smoke tests and the shipped helper's `--help`.
- Read-only mounting and detachment of the exact private mount.

A separate one-off read-only mount compared the actual icon resources to the
qualified source byte-for-byte. `Sheprd.icns` SHA-256 is
`2750da81af812d52c1a6853ab4dc4da45ee07a111f2943c7f0ad7e1a9655df8d`;
`Assets.car` is
`81154830ba48352b9826af7462a8608560ddd362c42ad9c93e1944532ac16536`.
These are the standard nebula assets; this hosted build has no development rim.

Local downloaded assets, metadata, receipts, notes and logs remain under the
task worktree's ignored `target/tagged-release-verification/20261110.4/`.
The public release assets are the lasting distribution route. The prior failed
`.3` tag and earlier debug previews remain unchanged.

This qualifies the requested signed macOS arm64 preview and tag workflow.
Native Dock/About/full app QA and broader runtime, update and production
acceptance gates remain open. No updater or other-platform release route was
enabled, and no personal application or runtime was installed or changed.
The local macOS 26 mount commands report their existing deprecation notice;
the hosted macOS 15 checks passed without that platform notice.
