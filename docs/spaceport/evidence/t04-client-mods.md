# T04 client mod removal

T04 and RM-MODS remain in progress. This cut removes the client-owned Luau mod runtime and its reachability. It does not pass A30 or remove the other excluded feature families.

## Ownership and graph

Before removal, `sheprd-mods` was a workspace member used by the GPUI app. `mods.rs` scanned a config folder and ran workers; those workers supplied status bar items, sidebar sections, palette commands, activity bar apps, and a right panel. The app exposed `[mods]`, `toggle_mod_panel`, and `sheprd mod` through its local control protocol. It bundled example mod folders and offered a managed `herdr-mods` skill. No separate installer or service hook for client mods was found in the release scripts.

The source modules, CLI and control protocol, command, config, UI places and saved panel width, English messages, examples, skill bundle, docs, workspace member, dependency edge and lockfile package are removed. `cargo metadata --format-version 1` shows `sheprd-mods` absent while `sheprd-luau` and `sheprd-automation` remain. Luau still serves automation, whose deletion is a separate unit. Ordinary sidebar, status bar, theme, font, layout and shortcut settings remain.

The app no longer installs or removes `herdr-mods`. It does not delete any skill already in a personal home: that file may belong to upstream Sheprd or to its author, and the fork has no ownership proof. Remaining exact `sheprd-mods`/`herdr-mods` references outside negative tests are in archival `design/sheprd-website` material; those inventories and the Spaceport deletion contract describe the old feature as source records, not reachable app behavior.

Herdr daemon plugin commands and the Settings > Plugins view remain reachable through the legacy Herdr 1 path. They are outside this client-mod runtime cut and are an explicit remaining exclusion gap for the legacy/plugin surface audit. No disabled mod loader is retained.

## Red and green checks

Before removal, the command catalog absence test failed on `toggle_mod_panel`; the CLI test failed because `mod` parsed; the config test failed because `[mods]` was accepted. The config test also asserts `terminal.size = 16` still loads after the old mod table is ignored.

- `RUSTUP_TOOLCHAIN=1.99.0 cargo test -p sheprd-commands --lib --locked` with `CARGO_INCREMENTAL=0`: 22 passed.
- `RUSTUP_TOOLCHAIN=1.99.0 cargo test -p sheprd-gpui --bin sheprd removed_mods --locked`: 2 passed.
- The already built GPUI test binary, `agent_skill::tests`: 16 passed; `preferences::tests`: 16 passed; `sidebar::activity::tests::layout`: 6 passed. The latter two used the cached binary without another Cargo build.
- Changed Rust sources passed `rustfmt --edition 2024 --check`; `just check-file-size`, `just check-sorted`, `just check-i18n`, and `git diff --check` passed.
- From `docs/spaceport`, `python tools/spec.py render`, `python tools/spec.py check --write-report`, and `python tools/test_spec.py` passed; 21 spec-tool tests. The report validates the package, not app conformance.

The GPUI test link still showed the pre-existing macOS compact-unwind warning, the pre-existing unused observation field, and Cargo's `block` future-compatibility notice. Native desktop QA, Linux/Windows checks, the full workspace suites, package binaries, and the remaining plugin/automation cuts were not run here. A30 stays `not_run`.
