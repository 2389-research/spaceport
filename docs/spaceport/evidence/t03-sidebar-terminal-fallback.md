# T03 sidebar terminal fallback regression

Source base: integration `9952c1efa`. The broad headless sidebar run failed
298/8. Seven failures shared one cause: `group_shown` treated an absent
browser key as an unavailable Core group even when the window selected a
legacy endpoint. The layout fixture has a snapshot with no focused workspace,
so it must still draw the terminal canvas and route ordinary sidebar input.
The new focused test first failed 0/1 (`Empty` versus `Terminal`), then passed
1/1 after the availability guard was scoped to selected Core groups. The
broader sidebar run improved to 305/1; the zero-endpoint Core tick still
passed 1/1.

The remaining scrolled device-drag failure had a different cause. At maximum
scroll, the last header's recorded top was 61 px while the scroll viewport
began at 62 px; its first pixels lay under the sticky header strip. A press
5 px below its top did not arm a drag. The test now starts two header heights
above maximum scroll and asserts the header is inside the viewport and the
press arms the drag. The focused scrolled drag passed 1/1, and the full
sidebar group passed 306/306. Product drag code did not change.

The production change preserves the existing terminal frame during a
transitional legacy connection with no focused workspace; it does not add a
second Core route or claim that Herdr 1 remains in the final product. The
exact Core-group availability check remains. Native macOS, Windows, Linux,
and live Core checks remain open. T03 stays `in_progress`, T04 stays
`blocked`, and A33 stays `not_run` pending native parity.

The all-target, all-feature app check passed with `--locked --offline`.
`just format-check`, `check-file-size`, `check-sorted`, `check-i18n`, package
integrity and its 21 tool tests, and `git diff --check` passed. The only
notices were the inherited macOS compact-unwind linker warning and pinned
`block v0.1.6` future-compatibility notice.
