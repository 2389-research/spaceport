# Reuse macOS build caches

Doctor Biz approved caching reusable work after reviewing the slow CI and release timings.
This covers unsigned build inputs and dependencies. Signing controls stay unchanged.
Do not cache credentials, keychains, submission receipts or release artifacts.

Baseline: release 43153449527 rebuilt both tools and the app without a release cache.
GitHub isolates tag caches. Seed reusable caches on trusted `main` instead.
The repository currently holds 17 caches, about 13 GB.
Cache dependency outputs rather than whole workspace builds.

- [x] Add runnable SDK preparation tests; observe failure before implementation.
- [x] Reuse validated SDKs outside Cargo's cleanup tree through one helper.
- [x] Share cache setup across preview builds and optimized macOS CI jobs.
- [x] Cache pinned helper binaries; use checksummed prebuilt downloads on misses.
- [x] Warm preview/arm64/x86_64 caches on trusted main when build inputs change.
- [x] Run native validators, SDK tests, existing release tests and quick checks.
- [x] Complete an independent trust-boundary review and fresh-eyes review.
- [ ] Push a PR, pass CI, then merge and verify real cold/warm runner behavior.

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
