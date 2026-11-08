# T04 terminal annotation removal

T04 and RM-ANNOTATION are in progress. This bounded cut removes terminal selection annotations and their send-to-agent path. It does not remove the other excluded feature families or pass A29.

## Reachability and retained paths

Before the cut, Shift selection offered a terminal annotation pill. `AnnotateSelection` reached the same composer through the command catalog, palette, key bindings, and shortcut settings. The window owned a note queue and rendered a composer; `terminal_notes` built prompts and `window/annotate` sent or copied them. The sample config, English messages, and GUI README described these paths.

The cut removes those terminal-only modules, wiring, catalog entry, shortcuts, messages, and product docs, including CLI help and the site's illustration copy. The config parser derives known bindings from the catalog, so an old `annotate_selection` binding becomes an unknown key and is ignored. Ordinary selection, Shift drag over a mouse-reporting pane, copy-on-select, retained copy, select-all, paste, and scrollback selection remain. `agent_notes`, browser feedback and review delivery are shared with other feature families and remain for their own T04 cuts. `selection_epoch` also protects links, accessibility, transfers, and images and remains.

No terminal-only crate dependency or build script was found, so the Cargo graph and lockfile did not change. The repository's specification and decision references to terminal annotations remain as the deletion contract.

## Test evidence

Tests were written first and failed for the expected reasons: the catalog still contained `annotate_selection`; config still accepted that key; Shift selection still drew the pill. The first command test attempt used the shell's Rust 1.97.1 and stopped before compilation. All later Rust commands selected the repository's installed 1.99.0 toolchain.

- `RUSTUP_TOOLCHAIN=1.99.0 cargo test -p sheprd-commands --lib --locked`: 21 passed.
- `RUSTUP_TOOLCHAIN=1.99.0 cargo test -p sheprd-gpui --bin sheprd terminal_annotations --locked`: 1 passed.
- `RUSTUP_TOOLCHAIN=1.99.0 cargo test -p sheprd-gpui --bin sheprd window::selection::tests --locked`: 30 passed, including Shift drag without an annotation pill and retained clipboard behavior.
- `RUSTUP_TOOLCHAIN=1.99.0 cargo test -p sheprd-gpui --bin sheprd shift_select_hint --locked`: 5 passed.
- `RUSTUP_TOOLCHAIN=1.99.0 just format-check`, `just check-file-size`, `just check-i18n`, and `just check-sorted`: passed.
- From `docs/spaceport`, `python tools/spec.py render`, `python tools/spec.py check --write-report`, and `python tools/test_spec.py`: passed; 21 spec-tool tests.
- `node --check site/assets/kit.js` and `git diff --check`: passed.

The GUI test link emitted the same macOS compact-unwind warning and Cargo `block` future-compatibility notice seen before the cut. Native desktop interaction, Windows and Linux compilation, the full workspace suites, and package contents were not checked in this slice. A29 and other T04 acceptance scenarios remain `not_run`.
