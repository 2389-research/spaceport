# Published Sparkle preview 20261117.1

This record was written on 2026-11-17, after publication, during the registry
reconciliation. The release was already public; nothing was built, signed,
submitted or published for this record.

The [prerelease](https://github.com/2389-research/spaceport/releases/tag/v20261117.1)
was published on 2026-11-17 from tag `v20261117.1`.
The tag resolves to source `723be121f3d6850968865d1735569fb7b0514892`, the merge of PR #90.
The [preview workflow](https://github.com/2389-research/spaceport/actions/runs/44173386473)
passed all five jobs: validate, ci-gate, build, sign and publish.

| Observation | Verified result |
| --- | --- |
| Public release ID | `449190946` |
| App identity / Team | `ai.2389.spaceport` / `HD9NM9NSMK` |
| Version / profile / architecture | `20261117.1` / release / arm64 |
| App notarization | `09b4f30d-01d4-434c-9d8c-4f6644146463`, Accepted |
| DMG notarization | `0460e906-7a9f-4fd2-929c-7b0be5622dd4`, Accepted |
| DMG | `Spaceport-20261117.1-arm64-apple-darwin.dmg` |
| Size | 32,963,495 bytes |
| SHA-256 | `9d8dcff2ff9942c7430503fd7e77546e99d07f1803f153ddab02fa38d28d963c` |
| Signed workflow artifact ID | `14370480510` |
| Workflow ZIP SHA-256 (GitHub's recorded digest) | `cd8316df0f2bbafacdc6263d2c36b738b5c3e95aedfece31225a22827152c683` |

## Public artifact verification

Downloaded the public DMG and `SHA256SUMS` with `gh release download`.
`shasum -a 256 -c SHA256SUMS` passed, and the digest matches GitHub's recorded asset digest.
Downloaded the signed workflow artifact with `gh run download`.
`cmp` confirmed identical public and retained DMG and checksum bytes.
Both retained Apple status records say Accepted.
The workflow ZIP digest above is the value GitHub reports; it was not recomputed locally.

`verify-macos-preview.sh 20261117.1 release` passed on the public DMG with the same-run metadata.
Strict DMG and deep app signatures passed. Both stapled tickets validated.
Gatekeeper accepted the app as `Notarized Developer ID`.
Identity, version and profile checks and the mounted executable's CLI smoke passed.
The private read-only mount detached.
The scripts came from `origin/main` at `98147d511`, on macOS 27.2.1.

## Feed

The public `appcast.xml` (SHA-256 `202497c84496632a1249c9631e6a1ecedca558f63b9d8d3b3f3c082e6f3e7496`) is byte-identical to the run's retained feed.
`sparkle-feed.py verify --archive` accepted the public feed and the public DMG.
The enclosure names version `20261117.1`, this release's URL and length 32963495.

## Not checked

The Sparkle `validate-bundle` probe and the old-namespace feed redirect were not rerun.
No native glyph, link or installed-update and relaunch check ran; those stay with Katas `6q5m` and `zafp`.
No personal app or runtime was installed or changed.
The downloads sat in a session scratch directory and were not retained.
