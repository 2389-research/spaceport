# Published Sparkle preview 20261113.1

This record was written on 2026-11-17, after publication, during the registry
reconciliation. The release was already public; nothing was built, signed,
submitted or published for this record.

The [prerelease](https://github.com/2389-research/spaceport/releases/tag/v20261113.1)
was published on 2026-11-13 from tag `v20261113.1`.
The tag resolves to source `64d51b05befd80a7930e09f5ecf8fe555cf45c7f`, the merge of PR #22 (Herdr Core opt-in, D69).
The [preview workflow](https://github.com/2389-research/spaceport/actions/runs/43479889079)
passed all five jobs: validate, ci-gate, build, sign and publish.

| Observation | Verified result |
| --- | --- |
| Public release ID | `444584278` |
| App identity / Team | `ai.2389.spaceport` / `HD9NM9NSMK` |
| Version / profile / architecture | `20261113.1` / release / arm64 |
| App notarization | `7db09d98-2304-4ae8-841e-f890cd5bdb8e`, Accepted |
| DMG notarization | `83bc3ddc-bd15-42dc-8c1c-002e2b1cf7c2`, Accepted |
| DMG | `Spaceport-20261113.1-arm64-apple-darwin.dmg` |
| Size | 33,683,124 bytes |
| SHA-256 | `272d7b7e5d4ca19e90c6e920e25455350f96042da344c270fe1804374a556d70` |
| Signed workflow artifact ID | `14065212091` |
| Workflow ZIP SHA-256 (GitHub's recorded digest) | `53bcfdb46f74d36110c34a036a10f489582822af6b15ab34023dc1715c25d526` |

## Public artifact verification

Downloaded the public DMG and `SHA256SUMS` with `gh release download`.
`shasum -a 256 -c SHA256SUMS` passed, and the digest matches GitHub's recorded asset digest.
Downloaded the signed workflow artifact with `gh run download`.
`cmp` confirmed identical public and retained DMG and checksum bytes.
Both retained Apple status records say Accepted.
The workflow ZIP digest above is the value GitHub reports; it was not recomputed locally.

`verify-macos-preview.sh 20261113.1 release` passed on the public DMG with the same-run metadata.
Strict DMG and deep app signatures passed. Both stapled tickets validated.
Gatekeeper accepted the app as `Notarized Developer ID`.
Identity, version and profile checks and the mounted executable's CLI smoke passed.
The private read-only mount detached.
The scripts came from `origin/main` at `98147d511`, on macOS 27.2.1.

## Feed

The public feed has since moved to `20261117.1`, so this version's feed entry was not checked.
The run's retained `appcast.xml` was not verified either.

## Not checked

The Sparkle `validate-bundle` probe and the old-namespace feed redirect were not rerun.
No native glyph, link or installed-update and relaunch check ran; those stay with Katas `6q5m` and `zafp`.
No personal app or runtime was installed or changed.
The downloads sat in a session scratch directory and were not retained.
