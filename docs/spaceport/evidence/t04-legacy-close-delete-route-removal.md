# T04: legacy workspace Close and worktree Delete route cut

Base: integration `8b48b7d48`. This bounded cut removes the app's legacy
Herdr 1 `workspace.close` and `worktree.remove` routes as whole routes. It does
not complete T04 or A29. Core document Archive/Delete, which edits presentation
metadata, is a different operation and remains guarded by target identity,
selection epoch, and document version. The layout-only group/tab close actions
remain too. No Core repository, personal daemon, credential, or real checkout
was changed.

The workspace popover no longer offers Close, Close group, or Delete worktree
checkout. Their dialogs, queued-removal state, force retry, path lookup,
keyboard command catalog entries, daemon key mappings, Settings preference,
config parsing, and error/locale text are removed. The independent Git close
preflight module is gone because no app request consumes it. `confirm_close_workspace`
is now an unknown config key. The archive script's alternate deletion path is
gone from the script enum, protocol enum, CLI parser, help, config schema,
trust question, and typed shell line. An old `[scripts] archive` entry is
rejected by the parser; Setup/Run retain their existing trust flow. Opening an
existing worktree, renaming a workspace, notes, ordinary file interactions,
and Core document actions remain.

TDD observations before implementation: `legacy_workspace_menu_cannot_close_or_delete_a_checkout`
failed because the w3 menu still drew `workspace-menu-Close group`.
`old_workspace_close_preference_is_reported_as_unknown` failed because the
parser returned no unknown key. The retained menu Rename control is asserted
in the first test. The script-line test now asserts that the only typed line
runs the reviewed script and contains no `worktree remove`; the config test
rejects `archive`, and the CLI test rejects `scripts archive`. Source search
after the cut found no `Method::WorkspaceClose`, `Method::WorktreeRemove`,
`WorkspaceAction::Close`, `WorkspaceAction::DeleteWorktree`,
`Command::CloseWorkspace`, `Command::RemoveWorktree`, `ScriptKind::Archive`,
`Script::Archive`, `queue_worktree_removal`, or `worktree remove --workspace`
in `crates/sheprd-gpui/src`, `crates/sheprd-commands/src`, or
`crates/sheprd-control-protocol/src`. The old method names remain as inert wire
definitions and tests in `sheprd-client`; this cut did not bulk-delete that
client crate.

`CoreWorkspaceDelete` and `Edit::Delete` remain in
`core_terminals/workspace_ui.rs`, with the document edit in
`core_terminals/workspace_document/edit.rs`; the same files retain Archive.
`group_menu::Action::CloseGroup` and `browser::close_group` still close only
window layout. Core document behavior must be rerun on the integrated branch
to prove terminal survival and exact version/target guards, not inferred from
the source search alone.

Verification: `just format-check`, `just check-file-size`, `just check-sorted`,
and `git diff --check` passed. Package-root `tools/spec.py check` passed
integrity validation and `tools/test_spec.py` passed 21/21 tool tests; these
do not prove application behavior. `just check-i18n` reports only the two
findings already present at the base (`menu/github.rs` literal “GitHub” and
unused `browser-no-pages`); it adds no route-specific unused message. A
scoped clean/rebuild of `sheprd-control-protocol` passed. The GPUI all-target
check could not establish this branch's build: agents sharing one Cargo target
from three worktrees produced stale cross-worktree metadata. The first check
saw removed `Script::Archive` and a missing retained `ToggleAnnotate`; after
a shared clean by another task, the second saw removed `Script::Archive`,
`Command::RemoveWorktree`, and `Command::CloseWorkspace`. Source in this
worktree has the opposite definitions. Stop here and rerun all-target compile
and focused red-to-green tests on the single integrated source tree. No native
UI check ran while macOS `loginwindow` was frontmost.

Remaining T04 dependencies: the local Git status/count worker still reads
checkouts through `menu/git.rs` and `sheprd-vcs`; the `git_status` sidebar
token and local count badge still consume those reads. GitHub account and
quota paths are being removed in a separate cut. The historical crate README
still documents other excluded feature families. A29 remains `not_run`; a
passing package validator or isolated compile is not product acceptance.
