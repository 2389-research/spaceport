# T01: named theme source isolation

Base integration revision: `f12aa286c`. Named custom themes were still found first under `<config root>/herdr/themes`. The GUI now looks first under `<config root>/spaceport/themes`; Ghostty theme locations remain available for explicit user customization. No upstream file was moved, edited, or read by this test.

The new regression uses a private temp root with different themes in `spaceport/themes` and `herdr/themes`. Before the code change, the test failed to compile because the injectable directory function did not exist. After the change, it found the Spaceport theme and excluded the upstream folder. The theme filter passed 14/14; config discovery passed 7/7. `just format-check`, `check-file-size`, `check-sorted`, `check-i18n`, and `git diff --check` passed. The test link emitted the existing macOS compact-unwind warning and Cargo reported the existing `block 0.1.6` future-compatibility notice.

The broader `config::tests::` filter ran 267 tests: 253 passed, 13 failed, one ignored. An exact retired-Boxd failure also reproduced on the unchanged `f12aa286c` integration base. Its assertion expects `load_startup_path` to reject a removed cloud table, while the current startup path calls a tolerant layered parser; the other failures include retired cloud settings and unrelated diagnostic assertions. This slice does not claim that suite passed or change those semantics. The failures need their own bounded correction.

The separate `Follow Herdr` choice still reads upstream settings, and native side-by-side installation was not run. T01 stays `in_progress`; A42 stays `not_run`; G-RELEASE remains open.
