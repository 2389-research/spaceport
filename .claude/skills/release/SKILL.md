---
name: release
description: Publish a Spaceport macOS preview. Use when the user asks to cut, push, ship, or publish a release or preview, or to check one that is already running. Covers the tagged `macos-preview.yml` route, its preconditions, and the changelog preview. No other release route exists in this fork.
---

# Releasing Spaceport

Spaceport has one release route: tagged, signed macOS arm64 previews, built by
`.github/workflows/macos-preview.yml` (decisions D61, D64, D65). The full
reference is the "Spaceport macOS previews" section of
`scripts/release/README.md`; read it before a release.

The inherited upstream production route is gone: `just release`,
`just release-beta`, `just release-promote`, the upstream release and
promotion workflows, Linux packages, Homebrew, winget, and the release
verification and GPG signing recipes were deleted. Restore them from git
history only when the general release gate opens
(`docs/spaceport/plan/gaps.json`). Do not improvise a replacement.

A preview goes out to real users: installed previews update through Sparkle.
Never push a release tag that the owner did not ask for.

## Preconditions

| Precondition | How to check |
| --- | --- |
| The commit belongs to `main` | `git merge-base --is-ancestor <sha> origin/main` |
| Main CI completed successfully on that exact commit | `gh run list --repo 2389-research/spaceport --workflow ci.yml --commit <sha>` |
| The version is unused, even by a failed run | `gh run list --repo 2389-research/spaceport --workflow macos-preview.yml` |
| The tag is pushed as `harperreed` | `gh api user --jq .login` |

The workflow enforces each of these and refuses otherwise. A failed check is a
stop, not something to work around.

## Show what will ship

Commit subjects are the release notes. Show them to the owner first:

```sh
just changelog-unreleased   # the commits since the last tag
just changelog              # the whole generated changelog
```

Both need `git-cliff` locally (`cargo install --locked --version 2.12.0 git-cliff`).
The workflow generates the published notes itself with
`scripts/release/generate-changelog.sh --macos-preview`. Do not hand-write notes
or edit a published body: fix the commit subjects instead.

## Tag and push

Versions are `YYYYMMDD.COUNTER`: the UTC date and a same-day counter that
starts at 1. Pick the next unused one, then push one annotated tag:

```sh
git tag -a v20261110.6 FULL_MAIN_COMMIT_SHA -m 'Spaceport macOS preview 20261110.6'
git push origin refs/tags/v20261110.6
```

The push builds an optimized app, signs and notarizes it, and publishes a GitHub
prerelease with the DMG and `SHA256SUMS`. It then replaces `appcast.xml` on the
fixed `macos-preview-feed` release.

A manual run on `main` (`workflow_dispatch` with an exact commit SHA and an
explicit version) uploads workflow artifacts and publishes nothing.

## Watch the run

```sh
gh run list --repo 2389-research/spaceport --workflow macos-preview.yml --limit 5
gh run view <run-id> --repo 2389-research/spaceport --json status,conclusion,jobs
```

The signing job uses the protected `macos-preview` environment. A job in
`waiting` may be waiting for a runner or for that environment; read the run
before telling the owner which.

A local `gh run watch` that dies on a network error says nothing about the run.
Re-check with `gh run view`.

## After a failure

Never rerun the workflow, and never move or delete a used tag to force another
submission. A used version stays used, even when the run failed before signing.
Inspect the retained notarization receipts, fix the cause, and release a new
version.

## Before touching release machinery itself

```sh
just release-check    # bash -n, mocked release tests, CI contract tests, actionlint, zizmor
```

It dispatches and publishes nothing. `just dmg VERSION` builds a local signed
and notarized preview DMG with the installed identity, and also publishes
nothing.

## Never

- Push a release tag the owner did not ask for.
- Reuse, move, or delete a version tag.
- Report success from a queue state, or failure from a dead watcher.
- Run an inherited upstream publish or install route.
- Edit the vendored protocol crate, the workflow pins, or `NOTICE.md` as
  incidental release cleanup.
