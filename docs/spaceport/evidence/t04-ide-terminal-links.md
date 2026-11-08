# T04: IDE links in terminal text

At `b59016d0`, the terminal link reader treated plain `vscode:` text as a clickable address and let an explicit OSC 8 `vscode:` link open another application without confirmation. This is an IDE-specific shortcut inside otherwise retained terminal and file-link behavior.

The new expectations failed first: `only_mail_addresses_open_without_asking` reported `vscode://file/tmp` as trusted. The implementation now limits automatic external-link trust to `mailto:` and removes `vscode:` from plain-text schemes. An explicit OSC 8 `vscode:` link still uses the generic unknown-scheme confirmation path, so ordinary terminal output remains visible and deliberate user clicks can still open a chosen link. Plain `vscode:` text stays text. The terminal engine and local file path handling did not change.

After the cut, `terminal::links::tests::external` passed 5/5 and `window::links::tests::cases::link` passed 41/41, including web, mail, file, unknown-scheme and the two revised `vscode:` cases. `cargo +1.99.0 check --locked -p sheprd-gpui --bin sheprd` passed on macOS with no code warnings; Cargo retained its existing `block v0.1.6` future-compat notice. `just format-check` and `git diff --check` passed. No native input or OS URL handler was exercised.

The code server, editor/review tabs, Git and GitHub graph are still reachable. This slice does not complete RM-IDE or A29. The explicit unknown-scheme confirmation is retained user permission behavior; removing it would weaken that boundary.
