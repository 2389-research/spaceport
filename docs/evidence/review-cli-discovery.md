# CLI discovery and caller contracts

Date: 2026-11-17.
Scope: Kata `x3b7`, `p3ft`, and `4a60`, under `9s44`.
Doctor Biz authorized implementation and validation.
Chat creation receipts (`j2p2`) require a separate lifecycle handoff.
No commits, pushes, PR changes, or Kata writes are authorized here.

## Accepted behavior

JSON errors retain the server's typed error code.
Exit statuses remain broader categories.
Collection responses use named JSON objects.
Empty tab, window, host, font, and integration lists exit 4.
The other catalogs retain their existing nonempty data and successful status.
Existing callers and tests consume the collection names explicitly.

Top-level help includes the shared options and exit statuses.
`help [COMMAND]` is available without starting the app.
Help and skill commands honor `--json`.
Local-only commands reject `--window` instead of ignoring it.
Hybrid commands preserve offline fallbacks only when no window was named.
A missing named window cannot become a local theme write or untargeted report.

`completions bash|fish|zsh` generates scripts from the shared family table.
Completion covers command families and shared options.
It does not claim command-specific parameter completion.
The scripts require no running app or runtime.

## Core origin evidence and guard

Pinned Core [api/run.rs](https://github.com/herdrdev/herdr-core/blob/cd12ddd88bc11490228343291dc1096c9ed3bf77/crates/herdr-core/src/api/run.rs#L212-L224)
sets `HC_TERMINAL_ID` and `HC_TERMINAL_SOCKET` for each terminal.
This was verified through the connected GitHub reader.
No daemon command or runtime inspection was needed.

The CLI detects either nonempty Core marker once when constructing its context.
It discards inherited legacy pane, workspace, and daemon qualifiers for that origin.
Global inspection commands remain available to discover window IDs.
App-window controls require an explicit window from Core callers.
Implicit targeting refuses with `NoWindow` and exit 5.

Legacy workspace notes, workspace mutes, process/port queries, Devices creation,
chat creation, online waits, and chat views without a named pane lack qualified Core targets.
They refuse with `Unsupported` and exit 8.
They never use a legacy focused workspace for the Core caller.
The guard neither starts nor queries Core.
Automatic Core context mapping remains unsupported.

## Regression evidence

Two initial JSON regressions failed before the change.
They reproduced compressed error identities and empty lists exiting 0.
A further catalog regression reproduced the same bare-array shape locally.
Executable socket peers also test distinct error codes and empty collection envelopes.

Three parser regressions failed before the help changes.
They reproduced missing `help`, missing completion commands, and ignored local window options.
The actual executable lacked the shared exit block in top-level help.
Its skill output also failed JSON parsing with `--json`.

Two initial Core executable regressions failed before the guard.
An implicit tab close reached the socket instead of refusing its missing target.
A note mutation reached the socket instead of refusing its missing Core mapping.
The implicit chat-pane regression reproduced the same focus fallback with a named window.

The named-window fallback regression observed successful local language output
when the explicitly requested window did not exist.
It now also covers config checks and refusal before a local theme write.

## Validation

Cargo uses Rust 1.99.0, `--locked`, the shared target directory, and the shared lock.

- Focused help parser tests: three passed.
- Focused control-client tests: 49 passed.
- Executable CLI suite: 39 passed, including target-boundary and offline fallback refusals.
- Full CLI parser tests: 60 passed.
- `cargo clippy --locked -p sheprd-gpui --tests -- -D warnings`: passed.
- `just check-file-size check-sorted`: passed.
- Scoped `git diff --check`: passed.
- Final help contract tests: three passed.
The integrating owner independently reviewed the guard, request paths, envelopes, help,
completion scripts, and offline fallbacks. No material findings remained.

Bash loaded the generated word completion specification.
Zsh loaded the generated completion function with its standard completion system.
Fish consumed the script and completed `spaceport comp` to `completions`.
These checks used generated artifacts in `/tmp/spaceport-qnz9/`.

Full workspace CI, all-feature gates, native Windows, and native Core-pane interaction remain deferred.
The existing `block v0.1.6` future-incompatibility notice remains.
The large test executable also reports the existing macOS compact-unwind size warning.

An unrelated existing limitation remains: an early-closing stdout pipe can make `println!` panic.
This was observed while piping a large theme catalog to `head`.
The integrating owner was notified; this batch does not expand into output-stream lifecycle work.
