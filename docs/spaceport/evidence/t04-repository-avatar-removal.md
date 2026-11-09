# T04 repository owner avatar removal

Source base: integration `23df7f8c0`. The sidebar's repository icon path
started `Avatars` workers in each window. They read local Git remote config or
queried saved SSH hosts, parsed a GitHub owner, then loaded a cached image or
fetched one from GitHub. This was reachable from ordinary workspace snapshots
even after account and PR UI removal.

The worker, local/remote Git config reads, GitHub image requests and disk
cache, avatar slot state, and GitHub mark asset are removed from the app
module graph. A small neutral folder SVG fills the same repository icon slot.
Non-checkout folders keep an empty slot for alignment, and standalone linked
worktrees keep their distinct branch glyph. The existing
`show_repository_icons` setting still hides this icon and gives the label its
room. Branch labels, notes, mutes, terminal/chat content, and workspace
selection are not changed. The old Octicons attribution file remains in the
repository; it does not load the removed asset.

## Reproduction and checks

The `turning_the_repository_icons_off_gives_the_name_their_room` headless
test first failed because the requested neutral `workspace-icon-herdr`
selector was absent (`/tmp/spaceport-t04-avatar-red.log`, 0/1). The existing
layout, folder-alignment, host-group, worktree-identity, and text-width fixtures
now target the neutral slot while checking the same bounds and on/off behavior.
The repository-icon tests passed 5/5 before the final dead-code cleanup;
worktree identity passed 7/7 after it. Host groups passed 2/2, plain-folder
alignment 1/1, and mark sizes 1/1. The headless aggregate text-width fixture
passed 1/1 after correcting a filter that matched no tests. The all-target,
all-feature app check passed with `--locked --offline`; `just format-check`,
`check-file-size`, `check-sorted`, `check-i18n`, package integrity and its 21
unit tests, and `git diff --check` passed. The first format check found one
changed signature needing rustfmt, which was fixed and rechecked. The usual
macOS compact-unwind linker warning and upstream `block` future-compatibility
notice appeared; no new source warning remained.

The `sheprd-github` crate remains reachable through separate worktree helper
and typed error paths. This cut does not claim full GitHub or worktree
deletion. T04 stays blocked, A29/A33 stay `not_run`, and native macOS,
Windows, and Linux view checks remain open. No personal checkout, saved SSH
host, installed app, cache, or GitHub account was accessed or changed.
