# T01/T04: pseudo-locale render fixtures

Base integration `7af7d2249`, under D58's allowed Herdr 1/Core policy. This
bounded test-only correction covers SP-001/SP-022 identity copy checks and
SP-002/SP-015/SP-016 excluded workspace actions. It changes no runtime,
translation catalog, native UI, daemon, or external service.

The workspace fixture opened the main checkout and then the linked checkout
with one recording text system. After excluded actions were removed, the linked
checkout's remaining labels all reused the main checkout's shaped text cache.
The recorder therefore saw no newly shaped translated message. The linked
checkout now has a separate test and fresh cache, while the original test
retains the main popover and all five supported dialogs. Both popovers require
Rename and Note controls and reject the retired worktree creation, close, and
delete controls. `assert_translated` remains unchanged.

The main-window fixture treated GPUI's measurements for the truncated Core
connection guidance as displayed text: single pseudo-locale letters and an
ellipsis prefix. Its shared check now uses the existing `uncut` helper already
used by the About and notice-card tests. Whole untranslated labels still reach
the original strict assertion. Input and notice tests reuse that window check.

TDD reproduced the four assigned failures before changing the fixtures:
`cargo +1.99.0 test --locked -p sheprd-gpui --all-features --bin spaceport
en_xa -- --nocapture` reported 47 passed and four failed. The first green
attempt exposed a new selector typo (`Note` instead of `Note...`), then a
revised fresh-context attempt exposed that calling `under_en_xa` twice on one
test thread toggles its shared locale back to English. The separate linked
test resolves the cache issue without changing that helper.

Final focused verification used pinned Rust 1.99.0 and the private APFS-cloned
`CARGO_TARGET_DIR=/tmp/spaceport-i18n-fixtures-target`:

- `cargo +1.99.0 test --locked --offline -p sheprd-gpui --all-features --bin
  spaceport en_xa -- --nocapture`: 52 passed, no failures or ignores.
- `cargo +1.99.0 test --locked --offline -p sheprd-gpui --all-features --bin
  spaceport -- menu::workspace_tests:: window::tests::`: 204 passed and two
  existing failures. The substring filter also selects `settings_window`;
  its two catalog-search tests still expect the removed
  `sidebar-change-counts` row. Those files belong to the separate integration
  cleanup and are untouched here.
- `RUSTUP_TOOLCHAIN=1.99.0 just format-check`, `just check-file-size`,
  `just check-sorted`, `just check-i18n`, and `git diff --check`: passed.
- Specification `tools/spec.py render`, `tools/spec.py check` with schema
  validation, and `python -m unittest discover -s tools -p test_spec.py`:
  passed, 21 tool tests. These prove package integrity, not application
  acceptance. T01 and T04 register this evidence without changing status.

The test link retains the existing macOS compact-unwind warning, and Cargo
retains the `block 0.1.6` future-compatibility notice. An initial unpinned Cargo
invocation selected inherited Rust 1.97.1 and stopped at the 1.99 MSRV check;
all subsequent Rust commands selected `+1.99.0` explicitly.

A fresh read of all five changed files found no runtime changes, weakened
global assertions, omitted retained dialog coverage, or broad untranslated
text allowances.

No full app/workspace suite, default-feature build, clippy, native desktop,
platform matrix, screenshot, personal daemon, or live API check is claimed.
T01/T04 status and all native acceptance gates remain unchanged.
