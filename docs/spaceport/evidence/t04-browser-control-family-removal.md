# T04 browser command family and reload socket removal

Source base: integration `72d2eeec9`. This bounded cut removes the entire
`spaceport browser` command family, its client/help/parser code and example
asset, and the separate `browser.reload` control request and response. The
control protocol rises from 32 to 33. The now unused reload-only window and
store helpers are removed. `spaceport skill`, shared `tab` commands and the
system URL handler remain. CLI docs, the managed skill, README examples and
the release CLI smoke expectation now match the command surface.

The negative parser and wire tests each failed 0/1 before implementation and
passed after it. The real executable rejects browser verbs before socket or
GUI startup in private test state; the retained skill command still prints.
The full control-protocol suite passed 62/62, the managed-skill group 16/16,
and the real CLI integration group 20/20. The app all-target, all-feature
check passed with pinned Rust 1.99.0, `--locked --offline`. `just
format-check`, `check-file-size`, `check-sorted`, `check-i18n`, package
integrity, its 21 tests and `git diff --check` passed. An initial package
test-discovery command pointed at a nonexistent `tests/` directory; the
package's actual tests live under `tools/`, where all 21 passed. The only
final Cargo notice was the inherited `block v0.1.6` future-compatibility
warning; the earlier focused
binary link also emitted the inherited macOS compact-unwind warning.

This does **not** close RM-BROWSER or A28. `TabOp` still exposes page
navigate/history/text/screenshot/zoom on the local control socket;
`browser/store`, the embedded page views, and `wry` still ship. Those routes
and the dependency graph require separate coherent cuts. No real browser,
Core daemon, native UI, Linux or Windows run occurred here. The system URL
handler and service-forward flow were preserved in source; this cut did not
retest live forwarding or prove G-PORT.
