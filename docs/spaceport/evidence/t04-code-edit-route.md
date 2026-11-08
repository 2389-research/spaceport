# T04/RM-IDE: remove the terminal editor control route

At source baseline `1769533a`, `sheprd code edit FILE[:LINE]` parsed through
`CodeCommand::Edit(FileAt)`, resolved a local file in the CLI, and sent
`TabOp::Edit` over the private control socket. The server checked the path and
caller workspace, then called `open_in_editor`, which can split a Herdr pane
and type an editor command. This was excluded IDE behavior. `code` had no
other verb or retained caller, so the whole CLI family and wire operation
were removed. Protocol version 17 records the shape change; an old `edit`
request now fails to decode instead of opening a pane. The command fails
with exit 2 before socket lookup. No compatibility alias or disabled handler
remains.

The parser regression first parsed `code edit` successfully; the wire
regression first decoded `{"op":"edit"}`. Both are green after removal.
Retained checks prove `tab list --window 42 --json` keeps the exact target,
the server still refuses unknown workspaces and invalid repository paths,
and Alt-click still opens a local file with the system handler. The terminal
editor module and its separate file-link, review and code-search callers
remain for later graph cuts. The installed agent skill had no `code edit`
promise; its negative test now guards both removed `code` verbs.

Verification in an isolated Cargo target: control protocol library 61/61;
focused GUI parser 2/2, server tab target 5/5, system file-link 1/1, local
folder resolution 1/1, agent skill 1/1; real-binary `code edit` and `code
open` refusal 1/1 each; production `cargo +1.99.0 check --locked -p
sheprd-gpui --bin sheprd` passed. The red tests failed on the expected
acceptance, not compilation. Existing macOS `__eh_frame` linker and
`block v0.1.6` future-compat notices remained. No real editor, Core,
personal daemon, user file or network service was used. Native desktop and
Windows/Linux checks were not run.

`just format-check`, `just check-file-size`, `just check-sorted`, `git diff
--check` and the package-root `tools/spec.py check` passed. The package
checker verifies specification integrity, not application behavior.

T04/RM-IDE remain in progress and A29 remains not run. This slice does not
prove the absence of all IDE/Git behavior or native UI parity.
