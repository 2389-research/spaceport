# Published About attribution preview 20261110.5

Doctor Biz approved the ownership wording, then explicitly requested merge to
main, push and a fresh tag. The qualified source is
`489f8c205dea3807d0d044fa4078b1586cd3bfcd`, selected by immutable tag
`v20261110.5`. [Full main CI](https://github.com/harperreed/spaceport/actions/runs/42907726241)
passed every required job, and the source gate independently confirmed it.
The [preview workflow](https://github.com/harperreed/spaceport/actions/runs/42909921543)
passed all five jobs and [published the prerelease](https://github.com/harperreed/spaceport/releases/tag/v20261110.5).

| Observation | Verified result |
| --- | --- |
| App identity / Team | `ai.2389.spaceport` / `HD9NM9NSMK` |
| Version / profile / architecture | `20261110.5` / release / arm64 |
| App notarization | `054f8a49-8edb-4862-b17a-d4a17a431b30`, Accepted |
| DMG notarization | `75188242-1de4-4113-bcb4-cc4c36a02b42`, Accepted |
| DMG | `Spaceport-20261110.5-arm64-apple-darwin.dmg` |
| Size | 32,215,223 bytes |
| SHA-256 | `7f9e3c1e7e55996684f413c51041d35de36d73268726973cc94ea06b04482b53` |

## Attribution and source verification

About identifies Spaceport by 2389 Research, Inc., distinguishes its
modifications copyright from original Sheprd work by Fabien Penso, and retains
both Sheprd and Herdr team credits. Spaceport's repository is the primary link;
upstream project links remain under credits. Apache-2.0 and third-party notices
have direct links. NOTICE adds a Spaceport header and retains its prior text.

The implementation is `68c5e6c6d`. Its first CI exposed one missed assertion:
the worktree PR banner uses the same repository constant, but its test still
expected the upstream URL. That failure reproduced locally. `489f8c205` keeps
the existing click/layout coverage and requires the correct Spaceport PR URL.
The corrected source passed full CI; no test or release gate was disabled.

Verification actually performed:

- Three attribution tests first failed against the original behavior. The
  added text then exposed a 600px-high layout overflow; tighter spacing keeps
  the OK button reachable. All eight About tests passed, including 360px width,
  project/license/notice links, closing behavior and pseudo-localization.
- Full local all-feature app unit tests passed 3,840 cases with nine existing
  opt-in ignores. Ten packaging/notice tests, strict app Clippy with all targets
  and features, formatting, i18n, source bounds, sorting and diff checks passed.
- Real native macOS profile/About checks passed at wide and narrow sizes.
  Actual painted text, wrapping bounds, Escape and focus were checked.
- Both Windows native CI jobs logged `ABOUT native PASS` for the corrected
  source; their UI and release performance checks completed successfully.
- The native paint inspector skips the ink bounds of Fluent's invisible FSI/PDI
  marks. Source text remains unchanged and every visible glyph must still
  match. The separate full sidebar bidi check confirmed those marks take no
  space and draw no missing-glyph boxes.
- Independent and fresh-eyes reviews found no remaining change blocker.

The broader macOS sidebar fixture fails later in host decoy activation with
`key=false`, `resized=true`, `app_active=false`, `app_hidden=false`. The unchanged
`7143eb821` baseline failed identically. This is recorded separately from the
successful About checks; the full local native suite is not claimed as passed.
Baseline debug linking also reports its existing unwind-table size warning,
and Rust reports the existing `block 0.1.6` future-compatibility notice.

Source verification logs remain in the task worktree's ignored
`target/about-attribution/`. General runtime, updater and full application
acceptance gates remain open.

## Verification of the published download

Downloaded the public DMG and `SHA256SUMS` after publication. The checksum
passed, and `cmp` proved the public image byte-identical to the retained signed
workflow artifact. The downloaded workflow ZIP matched GitHub's recorded digest,
`d3e347cdcffbc07d63099d1a9585d2db733d73a7085cdb2b8784c5055123036f`.
Both retained Apple submission status records say Accepted. No submission was
retried, and earlier tags/artifacts remain unchanged.

`verify-macos-preview.sh 20261110.5 release` passed locally on the public image
with its same-run metadata. Strict DMG and deep app signatures, exact identifier
and Team, both stapled tickets, and Gatekeeper passed with
`source=Notarized Developer ID`. Compiled version/profile/architecture identity,
the actual mounted executable's canonical CLI smoke and the helper's `--help`
passed. The exact private read-only mount was detached.

A one-off read-only resource check verified that the image's NOTICE matches the
new source byte-for-byte (SHA-256
`ee0b83d0e8ee71ca14d22acb44313ff9c285b2de70ba15f8dd26a49fa3afaefc`).
The binary contains the new publisher and attribution catalog messages.
`Sheprd.icns` and `Assets.car` match the approved standard nebula assets, with
SHA-256 `2750da81af812d52c1a6853ab4dc4da45ee07a111f2943c7f0ad7e1a9655df8d`
and `81154830ba48352b9826af7462a8608560ddd362c42ad9c93e1944532ac16536`.
These resource observations apply to the public download because its bytes match
the checked artifact exactly. This does not claim a full native UI run from the
signed image; the native About checks above exercised the qualified source.

The release body uses the shared pinned git-cliff generator, bounded by the
published `.4` ancestor. It includes the ownership fix and names source
`489f8c205dea3807d0d044fa4078b1586cd3bfcd`. Public assets, metadata, receipts,
notes and verification logs remain under `target/about-attribution/20261110.5/`.
The public release assets are the lasting distribution route. No personal app
or runtime was installed or changed. Local macOS 26 mount commands report their
existing deprecation notice; the hosted macOS 15 qualification passed.
