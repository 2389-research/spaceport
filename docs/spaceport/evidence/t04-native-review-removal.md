# T04/RM-GIT: native checkout review removal

At base `38f808e3`, the command catalog and group menu opened a client-owned
Git diff/review tab. `review.rs` and its `diff`, `view`, `notes`, and `viewed`
modules loaded local checkout changes, kept review state, and sent line notes.
This cut removes that full native route, its window polling/rendering, settings,
locale messages, icons, and active README instructions. It does not remove the
separate orchestrator PR dialog or checked system file opening.

Old saved `"kind":"review"` tabs still deserialize through a validated,
parse-only `RetiredReview` shape. The store drops them after calculating the
next tab ID, so a saved Core terminal or page keeps its ID and a stale group
pick cannot select a new tab. The new regression also checks that a relative
checkout path in an old review tab fails validation.
The control tab kind no longer accepts `review`; protocol version 22 records
that wire change. Rich-chat code highlighting now owns the pure syntax spans
formerly housed under `review::highlight`. Page `agent_notes` and checked file
links remain live.

The saved-tab test failed first against the old store: it restored the review
tab alongside the page and Core terminal. After the cut it passed. A new
control-protocol assertion rejects an old `"review"` tab kind. On macOS, the
command and control-protocol suites passed 25/25 and 55/55. Focused GPUI
filters passed: retired review tab 2, group menu 29, rich chat 5, checked
file links 7, agent notes 10, orchestrator review-request dialog 2, and
control tab routes 8. The production GUI check passed with no code warnings.
`just format-check`, `just check-file-size`, `just check-sorted`, and
`just check-i18n` passed (5,092 messages in 267 files). The existing macOS
compact-unwind linker and `block v0.1.6` future-compatibility notices remain.
The Spaceport package render/check passed and its 21 specification-tool tests
passed; that check validates the package, not this application's behavior.

T04 and RM-GIT/RM-IDE remain `in_progress`; A29/A30 remain `not_run`. GitHub,
checkpoint, worktree and issue workflows remain for separate deletion cuts.
The reachable `site/` marketing page still advertises native side-by-side
review and renders an illustrative review mock; T12 owns that stale content.
No native window, Windows/Linux build, real checkout diff, personal daemon,
account, user config, or Core service was exercised.
