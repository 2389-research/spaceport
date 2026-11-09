# T04 integrated sidebar test comparison

Integration `5c91205e2` contains the automatic Git reader, palette clone and
mockup-feedback removals. Both default and all-feature workspace `cargo check`
runs passed with Rust 1.99.0, `--locked --offline`. The control protocol suite
passed 61/61, project palette and opening tests 20/20, CLI integration 19/19,
and focused sidebar layouts 9/9, workspace marks 1/1, Git tick absence 1/1,
events 4/4, local notification feedback 2/2, removed page-note seam 2/2,
and control app targeting 8/8. The inherited macOS compact-unwind linker
warning and `block v0.1.6` future-compatibility notice remain.

The broad `sidebar::` filter was red. Running it on the mockup-feedback
worktree, which contains the feedback cut but precedes the Git reader and
palette cuts, gave 298 passed and 11 failed. Ten failing test names were
identical to the integrated run (296 passed, 10 failed); the extra failing
`mark_sizes` fixture tested a removed Git badge and was intentionally deleted
with that feature. This comparison shows the three new cuts did not introduce
the ten remaining failures. It does not prove they existed in pinned upstream.

One of those ten was a stale fixture from the earlier legacy Close/Delete cut:
`sidebar_allocates_text_width` still required a `Close group` tile. A red run
showed the missing selector. The fixture now asserts that the removed tile is
absent and checks the retained Rename tile and Open worktree row against their
current geometry. Its focused rerun passed 1/1. The broad rerun reached 297
passed and nine failed. Those nine are device drag (2), device footer count
(1), workspace row drag (1), update panel (1), workspace menu interactions
(2), and collapsed rail (2). The unchanged failures predate these three cuts,
but their relation to older Spaceport changes or upstream has not been
established.

The device-footer failure was a stale test count formula. `render_sidebar`
appends the Core workspace section once after the device loop, so two devices
produce two sets of device rows and one Core section. The previous assertion
doubled the entire local count, including that shared section. A focused red
run reproduced `[84, 4]` versus `[83, 4]`. The fixture now checks that Local
remains visible, the remote host disappears on Local scope, and the Core
section remains visible in both scopes; it doubles only device rows. The
focused rerun passed 1/1. A broad rerun using `cargo +1.99.0 test -p
sheprd-gpui --bin spaceport sidebar:: --locked --offline -- --nocapture`
reached 298 passed and eight failed. The remaining failures are device drag
(2), workspace row drag (1), update panel (1), workspace menu interactions
(2), and collapsed rail (2). Their root causes remain unproved, and the suite
is still red. The inherited compact-unwind and `block v0.1.6` notices remain.

No native UI was exercised during these runs. The Mac was locked, and
Windows/Linux native checks remain unrun. T04 remains blocked by G-LINK and
other excluded routes. A29 and A33 remain `not_run` pending complete negative
reachability and native interaction evidence.
