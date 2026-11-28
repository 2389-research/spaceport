# The per-function quality gate, and the sweep that removes its allowances

The workspace checks each Rust function in crates that inherit its lints.
This note records the November 2026 baseline and how to retire its exceptions.

## The gate

`just lint` (and every clippy step in CI) fails a function that is too tangled
or too long:

| Ceiling | Lint | Threshold | Where |
| --- | --- | --- | --- |
| cognitive complexity | `clippy::cognitive_complexity` | 25 | `clippy.toml` `cognitive-complexity-threshold` |
| length in lines | `clippy::too_many_lines` | 100 | `clippy.toml` `too-many-lines-threshold` |

Both Clippy lints are allow-by-default, so the root
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

At the integration with `5358a691c`, the macOS all-target, all-feature run reported
199 findings in 189 functions. Ten functions exceeded both limits. Each function
carries only the lint or lints it triggered, with its original measurement.
Linux enables the `log` feature on `tracing`. Its macro expansion also takes
four client workers or handlers, the control accept loop, and three app functions
over the complexity limit. Those exceptions apply only on Linux. Find the extra
diagnostics from macOS by adding `--features tracing/log` to the Clippy command.

| Crate | Complexity | Lines |
| --- | ---: | ---: |
| `spaceport-acp` | 1 | 5 |
| `spaceport-activation` | 0 | 1 |
| `spaceport-chat-core` | 0 | 1 |
| `spaceport-client` | 1 | 6 |
| `spaceport-commands` | 0 | 1 |
| `spaceport-core-client` | 1 | 5 |
| `spaceport-endpoint` | 0 | 1 |
| `spaceport-gpui` | 8 | 149 |
| `spaceport-live` | 1 | 2 |
| `spaceport-menu` | 0 | 4 |
| `spaceport-protocol` | 1 | 1 |
| `spaceport-settings` | 1 | 3 |
| `spaceport-settings-window` | 0 | 2 |
| `spaceport-sidebar` | 0 | 2 |
| `spaceport-textfield` | 0 | 2 |

## The allowances, and what each owes

The source attributes identify the remaining functions even when lines move.
Find the baseline with `rg '2026-11-27 baseline|2026-11-27 Linux' crates`.
Production, tests, and new code inherit the workspace gate. When a platform
alone triggers a finding, keep any exception on that configuration.

## Sweeping one crate

1. Remove the target function’s allowance and run Clippy for its crate with all targets.
2. Split the responsibilities that pushed it over the limit. Keep behavioral tests with the code they cover.
3. Run `cargo clippy --locked -p <crate> --all-targets --all-features -- -D warnings`
   and repeat with `--no-default-features`. Use the corresponding platform CI for conditional code.
4. Keep the source attribute removed once both checks pass. The census above records the original baseline.

On macOS the all-feature build needs the pinned Sparkle SDK. Set
`SPARKLE_FRAMEWORK_PATH` to the directory containing its `Sparkle.framework`.
