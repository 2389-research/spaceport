# T04 profile popup and account ownership

Source revision: integration `ee21e53af`. This is a read-only dependency
audit. It changes no app code, Core state, credentials or personal settings.
The earlier entry cut removed the application-menu sign-in row and stopped a
titlebar click from starting OAuth, but the popup and account reader remain.

## Reachable routes

- `titlebar.rs:39-44` still opens `Page::GitHub`. Its slot renders the selected
  GitHub profile avatar, account attention and API rate limit, or
  `icons/github.svg` (`titlebar.rs:120-123,211`). The profile slot geometry and
  popup focus are retained UX, but its displayed identity is GitHub-owned.
- `menu/chrome.rs:635-636` renders `render_github_auth`. That page renders
  account switching, device code, pasted token, account status and action
  buttons (`menu/github.rs:476-760`). `github_actions` and `github_action`
  (`menu/github.rs:249-315`) reach OAuth, the GitHub CLI, token submission,
  add/remove account, approval and sign-out. `github_key` (`:318-410`) exposes
  the same actions through keys, so hiding buttons would leave a live route.
- Opening the page also makes `poll_github_at` probe the GitHub CLI and recheck
  account access (`menu/github.rs:191-244`). `window.rs:606` calls it every
  tick. It syncs saved host/additional accounts, polls authentication state and
  scheduled profile loads, and updates avatars and PR quota state even when the
  page is closed.
- `menu/pr.rs:24-32,48-115,175-260` turns those profiles and tokens into PR
  lookup scope and cache work. `menu/workspace/popover.rs:288` and
  `menu/workspace.rs:727` show the workspace PR card/action; `sidebar/render.rs:453`
  paints cached PR badges. `menu/github.rs:139-149` selects host or main
  profiles; `menu/github/accounts.rs:112-130` adds extra account tokens.
- `config.rs:152,617-640` accepts `[github]`, while `config/github.rs` edits
  additional accounts. GUI config reload reinitializes every account and
  resets PR cache (`menu/settings.rs:260-272`). The saved-device removal flow
  separately offers GitHub credential cleanup
  (`menu/devices/host_menu.rs:318-350,652-684`). The app and VCS crates still
  depend on `sheprd-github`; VCS read probes import its PR `Input` and Git
  process helpers (`sheprd-vcs/src/git.rs:14`, `git/ahead.rs:10`).

## Cut dependency

Removing only the popup's sign-in/token/add/remove controls would keep account
polling and PR reads active while taking away sign-out and account removal.
Replacing it with a blank or apparently signed-in profile page would hide that
state from the person using it. This is not a safe bounded split and would
violate SP-015 and SP-022. No UI disablement or fake profile was implemented.

The next coherent cut should first remove the specialized PR worker/cache,
workspace card/action and sidebar badges, keeping ordinary workspace text and
sidebar geometry under regression tests. Then remove account polling, saved
account config/edits, GitHub CLI/OAuth/token/key routes and host cleanup in the
same cut that replaces `Page::GitHub` with a local profile shell. Keep the
titlebar hit target, popup focus, accessibility and layout, and show no claimed
account identity until a real retained owner supplies one. Extract any pure
Git read-probe helpers still used by `sheprd-vcs` before removing its GitHub
dependency. Do not erase or migrate personal credentials as part of this cut.

T04 remains blocked by G-LINK and unfinished GitHub removal. A29 and A33 stay
`not_run`; this source map is not a negative-reachability or native UI result.
The package validator passed schema/local-link checks and all 21 package tool
tests passed. No app tests, Cargo build or native platform check ran for this
source-only audit.
