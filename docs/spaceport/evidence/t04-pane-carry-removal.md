# T04/RM-TELEPORT: cross-host pane carry cut

At source `427f70e2b`, the pane menu offered remote machine rows beside
local pane destinations. Selecting a remote row opened `CarryPane`, whose
worker used `sheprd-remote::teleport::carry` to review and move an agent
session, checkout or new worktree. This cut removes the remote rows, review
page, worker, backend carry module, its live test, exclusive messages and
the now-unreferenced follow/progress presentation. It does not query Core or
start a replacement transport. The source root contains no older README
claim for this route to update.

Separate direct/type/string/config/test searches found no pane-carry CLI
command or saved setting. After the cut, `CarryPane`, `PaneCarry`,
`pane_carry`, `teleport::carry`, and the carry locale keys have no hits in
application crates, ordinary docs, skills or scripts. The remaining uses of
the English verb “carry” refer to unrelated data or drag behavior.

The new headless GPUI regression uses a local pane fixture plus a connected
`.invalid` SSH device fixture. Before the cut it failed because the picker
counted four rows instead of its two local destinations. After the cut it
passed and checked the rendered `pane-move-0` and `pane-move-1` bounds and
the absence of `pane-move-2`. The six existing local pane-move tests pass,
including stale destination and menu settlement checks. The retained local
move still uses the Herdr 1 CLI; T03 owns its Core migration. This cut does
not claim that old runtime path as approved parity.

The backend `teleport` module remains because `pane_move`, rich-chat
conversion, process inspection, dispatch host selection and Orchestrator
still call its host, snapshot, launch or job types. `teleport::fresh`, Git,
provision and error types likewise have other callers or are part of those
remaining excluded routes. They need separate caller-by-caller removal; this
cut does not treat the shared module name as proof that every route is dead.
Carry-only diagnostic variants and messages were removed from the shared
Teleport error enum after a source-wide constructor audit; errors needed by
the remaining host and job routes stay in place.

On macOS, the focused absence test passed 1/1 and retained local pane-move
tests passed 6/6. A wider pane-menu filter passed 17/19; the two existing
right-click mouse-routing tests fail at their menu-open assertions even when
run alone. Baseline comparison is pending, so this wider result stays red.
`cargo +1.99.0 check --locked --offline -p sheprd-gpui
--all-targets --all-features` passed with no code warnings; default check
passed after the final prune. The existing macOS compact-unwind linker note
and `block v0.1.6` future-compatibility note appeared. `just format-check`,
`just check-file-size`, `just check-sorted`, package `spec.py check` and all
21 package tool tests passed. `just check-i18n` still fails solely on the
pre-existing unused `browser-no-pages` key. `git diff --check` passed.

No actual Core, SSH host, agent, checkout, account, personal config or
service was touched. Native GUI and Windows/Linux checks were unavailable
locally. T04 remains blocked by G-LINK; A29 and A30 remain `not_run` because
other excluded routes are reachable and full negative reachability was not
proved. This is one bounded T04 cut, not product acceptance.
