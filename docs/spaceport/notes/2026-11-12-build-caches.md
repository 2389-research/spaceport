# Reuse macOS build caches

Doctor Biz approved caching reusable work after reviewing the slow CI and release timings.
This covers unsigned build inputs and dependencies. Signing controls stay unchanged.
Do not cache credentials, keychains, submission receipts or release artifacts.

Baseline: release 43153449527 rebuilt both tools and the app without a release cache.
GitHub isolates tag caches. Seed reusable caches on trusted `main` instead.
The pre-change snapshot held 17 caches, about 13 GB.
Cache dependency outputs rather than whole workspace builds.

- [x] Add runnable SDK preparation tests; observe failure before implementation.
- [x] Reuse validated SDKs outside Cargo's cleanup tree through one helper.
- [x] Share cache setup across preview builds and optimized macOS CI jobs.
- [x] Cache pinned helper binaries; use checksummed prebuilt downloads on misses.
- [x] Warm preview/arm64/x86_64 caches on trusted main when build inputs change.
- [x] Run native validators, SDK tests, existing release tests and quick checks.
- [x] Complete an independent trust-boundary review and fresh-eyes review.
- [x] Push PR #12, pass CI, then merge to main.
- [x] Verify real cold/warm runner behavior.

Use `.worktrees/release-caching` on `ci/release-caching`.
Only the main warmer writes shared caches. PRs and tagged previews restore them.
Keep signer and publisher cache-free. Keep source CI and archive verification intact.
Do not create a new release to test this change.

Success means real cache hits followed by passing CLI checks on GitHub runners.
Use actionlint and zizmor for workflow validation, not new source-string assertions.
Run existing app unit/integration tests through CI and real SDK producer/consumer tests locally.
Tag-specific timing remains unmeasured until the next requested release.

Five native SDK tests first failed with the missing helper, then passed.
Forty-five existing release/CI tests and quick checks passed.
Independent review caught the pinned rust-cache shared-key behavior.
The corrected shared-key includes compiler and SDK identifiers. The reviewer confirmed the fix.

The latest actionlint (1.7.12) rejects GitHub’s newer self-repository syntax.
Use the repository’s existing checked-out action style with targeted zizmor annotations.
Checkouts use workflow or validated source SHAs. Signer and publisher controls remain unchanged.
An earlier shell command masked the actionlint error; final checks now stop on failure.

PR #12 merged as `98c0b6be4019a7fa221c6de6179761172149ae30`.
Full PR CI passed in run [43234058427](https://github.com/2389-research/spaceport/actions/runs/43234058427).
The first main cache warmer is run [43237204011](https://github.com/2389-research/spaceport/actions/runs/43237204011).
Main CI passed in run [43237203931](https://github.com/2389-research/spaceport/actions/runs/43237203931).
The preview build and CLI smoke passed; its compiled dependencies were saved.
Its logs exposed a helper-tool cache warning despite the successful job.
The installer chose `.install-action/bin`, while the cache expected `.cargo/bin`.
Stage exact tool binaries in a fixed runner directory and check their versions there.
The PATH entry contains only the runner's fixed directory, never workflow input or tool output.
Four real executable-staging tests cover cold use, warm reuse, missing tools and usage.
The tool-path fix and final cache-scope verification follow below.

PR #13 merged the tool-path fix as `61bb5d02c2329b6910dcb4d3722b94bfef86e837`.
Its full CI passed in run [43239926438](https://github.com/2389-research/spaceport/actions/runs/43239926438).
Both optimized PR jobs restored exact main cache matches and passed 22 CLI tests each.
Intel app compilation fell from 7m19s cold to 2m32s warm.
Apple Silicon app compilation fell from 8m55s cold to 4m19s warm.

The next main warmer, [43242788144](https://github.com/2389-research/spaceport/actions/runs/43242788144), saved the tool cache.
Its preview failed before compilation because the cache restored an empty packaging output directory.
Rust-cache cleans non-Cargo files recursively but leaves directories behind.
Cache `target/release` and explicit macOS target release profiles, rather than all of `target`.
The pinned action still cleans workspace artifacts before saving those selected paths.
The cache path change required the fresh seed verified below.

PR #14 merged the cache-scope fix as `cdbb5512857d830c0022295a61dddb7ba277d5a0`.
Its config-only CI passed in run [43243996895](https://github.com/2389-research/spaceport/actions/runs/43243996895).
Repository input detection skipped workspace builds for that config-only diff.
Full main CI for the tool-path fix passed in run [43242787879](https://github.com/2389-research/spaceport/actions/runs/43242787879).
The final cache seed passed in [43244372052, attempt 1](https://github.com/2389-research/spaceport/actions/runs/43244372052/attempts/1).
All three jobs built real executables and saved their selected dependency paths.
Both architecture jobs passed 22 CLI tests each. The preview passed CLI smoke and packaging checks.
Current-main config CI passed in run [43244371443](https://github.com/2389-research/spaceport/actions/runs/43244371443).
Only the preview was rerun as attempt 2, job `131285196116`.
The [warm preview](https://github.com/2389-research/spaceport/actions/runs/43244372052/attempts/2) passed.
Its logs confirm exact dependency, SDK and helper-binary cache hits.
The checksummed installer skipped downloads, and restored tool versions passed validation.
Compilation fell from 10m46s cold to 3m36s warm on the same source revision.
CLI smoke and unsigned bundle checks passed, without cache-path warnings or output collisions.
The selected dependency profiles, SDK and helpers total about 1.84 GB.
Older cache entries remain subject to normal GitHub eviction; storage settings stayed unchanged.

Retained logs live under `target/release-caching/` in the task worktree: `final-seed.log` and `final-warm.log`.
No release tag was created. Tag-specific timing remains unmeasured until the next requested release.
