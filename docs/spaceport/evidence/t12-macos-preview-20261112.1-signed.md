# Published Sparkle preview 20261112.1

Doctor Biz authorized the repository move, Windows build pause and fresh signed release.
D66 records that scope. The repository now belongs to `2389-research`.
Its ID, history, releases and protected signing environment stayed intact.

The [prerelease](https://github.com/2389-research/spaceport/releases/tag/v20261112.1)
uses source `e874b260c9d71f75cb5f3f30418546f810b10417` and immutable tag `v20261112.1`.
[PR CI](https://github.com/2389-research/spaceport/actions/runs/43150221075)
and [exact-main CI](https://github.com/2389-research/spaceport/actions/runs/43151863635) passed.
The real release gate accepted that exact main commit.
The [preview workflow](https://github.com/2389-research/spaceport/actions/runs/43153449527) passed all five jobs.

| Observation | Verified result |
| --- | --- |
| Public release ID | `442502733` |
| App identity / Team | `ai.2389.spaceport` / `HD9NM9NSMK` |
| Version / profile / architecture | `20261112.1` / release / arm64 |
| App notarization | `796cf137-8520-4169-aeb7-24b4e2d32590`, Accepted |
| DMG notarization | `a69fe78d-d22a-48d6-a85d-20f0a4eba129`, Accepted |
| DMG | `Spaceport-20261112.1-arm64-apple-darwin.dmg` |
| Size | 34,023,180 bytes |
| SHA-256 | `e8a0482bb541741571b00f63b32b25e2dba378b613c66aa1e70a7a0fd0db6fd9` |
| Signed workflow artifact ID | `13916340833` |
| Workflow ZIP SHA-256 | `03661db2a4cd0e2f81e9bc2cd582d629b26234806f8a8567f74e119b2cf52bf3` |

## Build recovery

PR #10 updates active repository links, CI conditions and preview/feed publishing routes.
It pauses Spaceport Windows check, native and package jobs under D66.
The aggregate gate requires their skipped state and successful active platform checks.
Windows performance budgets and eventual acceptance requirements stay unchanged.

The old queued preview exposed a separate release-history bug.
GitHub retained its commit-subject title before initializing the workflow run name.
The history check rejected that record and blocked fresh versions.
It now reserves valid push tags using their full source SHA.
Malformed or uncertain records still refuse. Twelve history tests and live history validation passed.
No old version was reused. No notarization submission was retried.

## Public artifact verification

Downloaded the public DMG, checksum and signed feed after publication.
Both release assets matched GitHub's recorded digests.
The signed workflow ZIP matched its digest before bounded extraction.
`cmp` confirmed identical public and retained DMG, checksum and feed bytes.
Both retained Apple status records say Accepted.

`verify-macos-preview.sh 20261112.1 release` passed on the public DMG with same-run metadata.
Strict DMG and deep app signatures passed. Both stapled tickets validated.
Gatekeeper accepted both as `Notarized Developer ID`.
Compiled identity/version/profile checks and the mounted executable's canonical CLI smoke passed.
The bundled chat helper's `--help` passed. The private read-only mount detached.

A throwaway read-only probe mounted the same public DMG separately.
The canonical Sparkle `validate-bundle` check passed for arm64.
Its actual bundle metadata names the transferred feed and unchanged Spaceport public key.
Signed feeds and pre-extraction archive verification are enabled.
The signed-feed failure expiration is numeric zero. Automatic installation is disabled.
The probe detached its exact private mount.

The canonical feed verifier accepted the published feed and actual DMG signatures.
The enclosure names version `20261112.1`, the transferred repository URL and the correct archive length.
The old `harperreed/spaceport` feed URL returned HTTP 200 after two redirects.
Its bytes matched the published feed exactly.
Keep the old namespace unused so those redirects remain available.

Pinned git-cliff 2.12.0 generated the reviewed notes from published ancestor `v20261110.5`.
The notes identify the qualified source and include Sparkle plus the branding and release fixes.
Artifacts and logs remain under the task worktree's ignored `target/release-org-move/20261112.1/`.
No personal app or runtime was installed or changed.

## Remaining checks

Windows performance remains unresolved in Kata `h8g5`; keep Windows builds paused until qualification passes.
The locked desktop prevented native link/glyph checks and a public installed-update/relaunch test.
Test Check for Updates from the installed baseline before claiming that route fully qualified.
General Windows/Linux packaging and full T12 acceptance remain open.
Local macOS mount commands emitted their existing deprecation warnings.
