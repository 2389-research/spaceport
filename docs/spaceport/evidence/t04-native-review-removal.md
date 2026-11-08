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

## Merged review and ID persistence follow-up

The merged control protocol is version **23** for the review tab-kind removal.
Version 22 removed the orchestration open route first; the version 22 statement
above describes the isolated branch before that merge. At merged commit
`bea2c1a3`, the control-protocol suite passed 56/56, the command suite 25/25,
the saved-review tests 2/2, and the all-feature app check passed. The merged
format, file-size, sorted, and i18n checks passed (5,067 messages in 266 files).

Fresh source review found that a later save discarded the in-memory retired-ID
reservation. A two-restart regression failed: a new tab could take the retired
review tab's ID. Commit `584a5d5a` persists the next tab ID in the saved state;
older state files default that field to zero and still derive the next ID from
their tabs. The same commit corrects `tab --help`, which still listed reviews.
The two new tests failed before their fixes and now pass. The saved-tab suite
passed 20/20 and the help test 1/1. Default and all-feature app checks,
`just format-check`, `just check-file-size`, and `git diff --check` passed with
no new code warnings. The existing macOS compact-unwind linker and `block`
future-compatibility notices remain. This verifies the serialized state logic,
not native group layout behavior while disconnected; A29 remains `not_run`.
