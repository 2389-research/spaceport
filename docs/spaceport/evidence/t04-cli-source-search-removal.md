# T04/RM-IDE: local checkout search command removal

At `28739a92`, `sheprd search symbol|file` still parsed without a running app.
Its local worker built `sheprd-vcs::code_index` over the caller's Git checkout,
then ranked files or definitions. The native Go to File/Symbol picker and
read-only source tabs were already gone. Sidebar name filtering and terminal
scrollback search use separate code and remain.

The new CLI test first failed because `search symbol open_workspace` produced
a valid `ControlCommand::Search`. The command family, parse/help/client path,
local indexing worker, unused GUI ranking module, and CLI docs row were then
removed. `search symbol`, `search file`, and `search --help` now return the
unknown-command error before any app socket lookup. A retained `processes
--pane pane-7` command still parses with its exact pane target. The
`sheprd-vcs::code_index` library module and its error mapping remain compiled
as dead code until the separate VCS crate cut; this change does not claim the
whole Git/IDE feature family is absent.

The focused GUI test passed after the removal. Its first green run found an
unused `code_index` import in the app root; removing that import made the
all-feature production check and rerun of that test pass without a new code
warning. Ten adjacent CLI parser tests and three retained local process
command tests also passed from the same test binary. `just
format-check`, `just check-file-size`, `just check-sorted`, `just check-i18n`
(5,181 messages in 271 files), and `git diff --check` passed. The test link
reported its existing macOS compact-unwind warning; Cargo reported its
existing `block v0.1.6` future-compatibility notice. Native UI, real CLI
process, Windows/Linux and full workspace checks were not run in this slice.
T04/RM-IDE stay in progress and A29 stays `not_run`.
