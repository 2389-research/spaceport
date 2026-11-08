# T01/T09 control command copy cut

Base: integrated `wip/spaceport-t00` at `40f6423b`. This cut changes the
user-facing product name in 38 runtime strings under
`crates/sheprd-gpui/src/control`. That includes CLI status and diagnostics,
settings text, app action replies, and window, tab, host, and work refusals.
The Rust `SheprdWindow` type, control wire types, internal paths, and literal
Herdr attribution stay intact. The existing exact-window response keeps its
`NotFound` code and echoes only the requested window ID.

`scripts/release/tests/test_control_product_copy.py` checks runtime control
string literals without treating comments, test fixtures, or type names as
product copy. It failed on 37 old labels before the edit. Tightening the
pattern to catch an escaped newline found one more old label and failed again;
it passes after the 38th edit. The focused Rust updater-copy test failed with
`update from Sheprd` before the edit. Existing Rust fixtures now assert
Spaceport in updater, language, shortcut, and missing-window output.

## Checks on this branch

- `python3 -m unittest scripts.release.tests.test_control_product_copy`:
  one passed after the red runs.
- `just check-file-size` and `just check-sorted`: passed.
- `cargo +1.99.0 fmt --all`, `just format-check`, and `git diff --check`:
  passed after fixing the one fixture layout reported by the first format run.
- `docs/spaceport/tools/spec.py check` with the existing package virtualenv:
  passed after linking this evidence. Plain `python3` could not run the schema
  check because `jsonschema` was absent in that interpreter.
- Focused Rust tests and production check after the edit: **not run** here.
  The first red GUI test link used the warm target and left about 8.5 GiB free;
  another link risks the shared 8 GiB floor. Run them on the integrated branch
  after disk is reclaimed. The red link printed the existing macOS
  compact-unwind warning and `block` future-compatibility notice.

The CLI help and `docs/cli.md` already use `spaceport` in this base; no help
or locale key needed changing for these direct Rust response strings. A full
product-copy review remains open: source search still finds 81 `Sheprd` lines
in the English Fluent catalog, including desktop UI copy. Production app,
package, publisher, repository, and updater identity remain gated by
G-RELEASE. T01/T09 remain in progress; A42 remains not run. This source and
headless fixture check does not prove native identity or release parity.
