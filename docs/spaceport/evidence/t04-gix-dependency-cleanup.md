# T04 Git parser dependency cleanup

Integration base: `72ba1e507`. The worktree script cut removed the last
`gix` use from application source, but its member and workspace manifest pins
remained so that cut could build with `--locked`. A search of `crates/` source
found no `gix` references; both remaining manifest entries were direct app
dependencies.

The two pins are removed and `Cargo.lock` is regenerated with pinned Rust
1.99.0 in offline mode. The lockfile drops 52 packages and adds none. The
surviving package names and versions do not change. This keeps Git parsing
out of the app dependency graph while preserving the separate developer Git
workflow and other retained product paths.

`cargo +1.99.0 metadata --format-version 1 --offline` and
`cargo +1.99.0 check -p sheprd-gpui --all-targets --all-features --locked
--offline` passed on the combined integration source. Only the inherited
`block v0.1.6` future-compatibility notice appeared. Focused integrated
headless checks passed: repository icons 3/3, zero-endpoint Core tick 1/1,
script-free workspace menu 1/1, and retained chat start 11/11. `just
format-check`, `check-file-size`, `check-sorted`, `check-i18n`, package
integrity and its 21 tool tests, and `git diff --check` passed. This evidence
does not prove T04's full negative reachability or native UI parity; T04 and
A29 remain open.
