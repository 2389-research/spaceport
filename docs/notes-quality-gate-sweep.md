# The per-function quality gate, and the sweep that removes its allowances

Measured 2026-11-27 on `origin/main` at `925370ab1b`. This note is the durable
record the sweep works from: the gate's mechanism, the census it produced, the
files that carry a dated allowance because of it, and what each one owes.

## The gate

`just lint` (and every clippy step in CI) fails a function that is too tangled
or too long:

| Ceiling | Lint | Threshold | Where |
| --- | --- | --- | --- |
| cognitive complexity | `clippy::cognitive_complexity` | 25 | `clippy.toml` `cognitive-complexity-threshold` |
| length in lines | `clippy::too_many_lines` | 100 | `clippy.toml` `too-many-lines-threshold` |

Both are clippy *restriction* lints, allow-by-default, so the root
`Cargo.toml`'s `[workspace.lints.clippy]` is what turns them on for every crate
that inherits the workspace lints (`cognitive_complexity = { level = "warn",
priority = 1 }`, and the same for `too_many_lines`). Clippy then passes
`-W clippy::cognitive_complexity -W clippy::too_many_lines` to rustc, and the
`-D warnings` the recipes already pass turns both into errors. There is no
separate recipe and no separate CI step: adding the entries added the gate.

Cognitive complexity counts nesting depth rather than branch count, so a long
flat `match` stays cheap while branches inside loops inside branches do not.
25 is the tangle line; 100 lines is about two screens. Both are a starting
value, not a doctrine: they reach the code this repository already writes
without demanding a rewrite in the change that added them, and the sweep
lowers them once the worst functions are split.

A file's 1,000-line ceiling is separate and unchanged:
`scripts/check-file-size.sh`, run by `just check-file-size`. These two bound the
function inside a file. There is no mutation-testing gate; the owner declined
one.

## The census

`cargo clippy --locked --workspace --all-targets --all-features` on the commit
above: **196 findings in 163 files** — 14 over the complexity ceiling, 182 over
the line ceiling. Nothing else warned. CI's ubuntu clippy found one more (see
the control-protocol row below): 197 all told.

| Allowance sits in | cognitive | lines | worst cognitive | worst lines |
| --- | --- | --- | --- | --- |
| `crates/spaceport-gpui` | 8 | 147 | `src/menu/chrome/keys.rs:26` 43 | `src/sidebar/render/rows/workspaces.rs:42` 390 |
| `crates/spaceport-client` | 1 | 5 | `src/session.rs:315` 27 | `src/session.rs:315` 236 |
| `crates/spaceport-core-client` | 1 | 5 | `src/tests/live/machines.rs:47` 37 | `src/tests/live/machines.rs:47` 185 |
| `crates/spaceport-acp` | 1 | 5 | `src/tui/view.rs:284` 28 | `src/tui/view.rs:284` 123 |
| `crates/spaceport-settings` | 1 | 3 | `src/herdr_settings/tests/theme_palettes.rs:4` 27 | `src/preferences.rs:105` 186 |
| `crates/spaceport-live` | 1 | 2 | `src/state.rs:482` 32 | `src/state.rs:482` 235 |
| `crates/spaceport-menu` | 0 | 3 | — | `src/chrome/panel.rs:73` 121 |
| `crates/spaceport-sidebar` | 0 | 2 | — | `src/row/paint.rs:106` 223 |
| `crates/spaceport-settings-window` | 0 | 2 | — | `src/ui/swatches.rs:38` 138 |
| `crates/spaceport-protocol/tests/surface_encodings.rs` | 1 | 0 | `:416` 32 | — |
| `crates/spaceport-protocol/tests/compatibility.rs` | 0 | 1 | — | `:143` 101 |
| `crates/spaceport-activation` | 0 | 1 | — | `src/fence.rs:136` 141 |
| `crates/spaceport-commands` | 0 | 1 | — | `src/lib.rs:43` 180 |
| `crates/spaceport-client/tests/live.rs` | 0 | 1 | — | `:280` 128 |
| `crates/spaceport-endpoint` | 0 | 1 | — | `src/host/link.rs:168` 183 |
| `crates/spaceport-chat-core` | 0 | 1 | — | `src/prepared.rs:430` 107 |
| `crates/spaceport-control-protocol` | 1 | 0 | `src/socket.rs:131` 26† | — |

† Found only by CI's clippy (ubuntu); the local 1.99.0 run scored this
accept-loop closure at or under 25. Added 2026-11-27 as a dated scoped
allowance after the first CI run; the count above reads 196 local findings,
197 with this one.
| `crates/spaceport-textfield` | 0 | 1 | — | `src/search_input.rs:450` 216 |
| `crates/spaceport-gpui/tests/live_gui.rs` | 0 | 1 | — | `:280` 114 |

The 34 findings in test code (`*/tests/*` paths and the two integration-test
files) get the looser treatment the repository already gives tests: where a
crate has no production finding — `spaceport-core-client` alone — the
allowance is on its `#[cfg(test)] mod tests;` declaration rather than the crate
root, and the opt-in integration-test files carry their own file-level
allowance next to the `unwrap_used` one they already had. Test code is still
counted, just not held to the production ceiling yet.

The fourteen functions over the complexity ceiling, worst first — these are the
sweep's first targets, because a tangle costs a reader more than length does:

| Function | Complexity |
| --- | --- |
| `crates/spaceport-gpui/src/menu/chrome/keys.rs:26` | 43 |
| `crates/spaceport-gpui/src/window/prefix.rs:224` | 39 |
| `crates/spaceport-core-client/src/tests/live/machines.rs:47` | 37 |
| `crates/spaceport-protocol/tests/surface_encodings.rs:416` | 32 |
| `crates/spaceport-live/src/state.rs:482` | 32 |
| `crates/spaceport-gpui/src/sidebar/layout_tests/text_width/sidebar_rows.rs:52` | 32 |
| `crates/spaceport-gpui/src/terminal/tests.rs:154` | 32 |
| `crates/spaceport-gpui/src/menu/chrome/render.rs:122` | 30 |
| `crates/spaceport-gpui/src/settings_window/native/fonts.rs:52` | 29 |
| `crates/spaceport-acp/src/tui/view.rs:284` | 28 |
| `crates/spaceport-gpui/src/chat/preview.rs:357` | 28 |
| `crates/spaceport-client/src/session.rs:315` | 27 |
| `crates/spaceport-settings/src/herdr_settings/tests/theme_palettes.rs:4` | 27 |
| `crates/spaceport-gpui/src/settings_window/native.rs:100` | 27 |

Of the 182 long functions, 70 are between 101 and 120 lines, 54 between 121 and
150, 34 between 151 and 200, and 24 over 200: most are a small split, and the
tail is the work.

## The allowances, and what each owes

Each is a `#![allow]` at the file or module that names the finding, with the
date and the count it accounts for. Nothing widens a lint for the whole
workspace.

`crates/spaceport-gpui` is the exception in size and in kind: 155 findings in
one crate. Its allowance shrinks as its modules are split, and it is deleted
when the crate's last finding is gone.

## Sweeping one crate

1. Split the function: extract the branches a name can carry, or move a stage of
   a long one into its own function. Do not raise the threshold, and do not
   paraphrase the work into a helper that only forwards.
2. Delete the crate's `#![allow]` (or just the lint it names, where the other
   still has findings) and run
   `cargo clippy --locked -p <crate> --all-targets --all-features -- -D warnings`.
   What it now reports is what is left to split; while the allowance is in place
   the same run is silent, so this is the only way to see progress.
3. Delete the crate's row from this note in the same commit as the allowance.

`spaceport-gpui` needs `SPARKLE_FRAMEWORK_PATH` on macOS — acquire it with
`python3 scripts/release/sparkle_framework.py acquire target/sparkle-sdk` —
because `--all-features` builds the preview updater. The census above was taken
that way.
