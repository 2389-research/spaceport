# T04: remove named Herdr 1 launch flags

Source baseline `13dc4573` still exposed top-level `sheprd --session NAME`
and `sheprd --dev`. `cli.rs` parsed these into `ConnectTarget::Session`, and
`connection.rs` passed that target to `daemon::connect`, which starts
`herdr --session NAME server` if its socket is absent. This is a Herdr 1
session route, not a Core machine identity. The plain launch still selects
`ConnectTarget::Local`; an explicit `--socket` remains for native fixture
tests. The installed-Core hub, terminal worker and emulator were untouched.

The new parser regression first failed: `--session` produced
`MissingSessionName` instead of an unknown-option refusal. After removal,
top-level `--session` and `--dev` refuse before GPUI startup. The release CLI
smoke check also requires both flags to be absent from help and rejected at
the parser boundary. The existing explicit non-UTF-8 socket path test remains.

`cargo +1.99.0 test --locked -p sheprd-gpui --bin sheprd cli::tests::`
passed 50 tests after one stale assertion was corrected. The first green run
had 49 passes and one stale `--dev` expectation; the rerun passed 50/50.
`python3 -m unittest scripts.release.tests.test_smoke_cli` passed 4 tests
after a pre-existing mock response for removed `browser open` was aligned
with the current smoke contract. `just format-check`, `just check-file-size`,
`just check-sorted`, and `git diff --check` passed. The GUI test link showed
the existing macOS `__eh_frame` warning and Cargo's existing `block v0.1.6`
future-compat notice. Native desktop, Windows/Linux, and full workspace
checks were not run.

RM-LEGACY remains in progress. The default Herdr 1 bridge and startup,
explicit socket route, in-app named-session UI, `window new --session`
control command, and legacy snapshot consumers remain reachable. They need
Core-backed replacements or their own bounded removal units before A30 can
pass. This cut does not treat a disabled route as Core parity.
