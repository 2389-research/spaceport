# T04/RM-PROVIDERS: Daytona dispatch creation cut

At `2a0e3c22`, the Daytona Settings creation card was already gone, but
`cloud::providers::provision(Daytona)` still exposed `daytona::create::provision`
to Smart Dispatch. That callback could create a sandbox and install Herdr.
This cut removes the callback, the direct Daytona creation helper, its
sandbox-create REST call and the orphaned setup job. The existing account
read/list, saved-device connection and delete code remains for a separate
provider removal cut. No provider request or saved device was used during this
change.
The device picker now describes management in Settings instead of offering
creation there.

The new dispatch test first failed because `provision(Daytona)` returned a
callback, then passed after removal. Daytona's retained loopback HTTP tests
passed 3/3; the old create test now checks that listing sends the saved key and
organization header. The provider test passed 1/1. Default and all-feature
`sheprd-gpui` production checks, `just format-check`, `just check-file-size`,
and `git diff --check` passed with no new code warnings. Cargo still reports
the existing `block v0.1.6` future-compatibility notice. A narrow remote-crate
test command with `--features daytona` failed to compile because unrelated
image tests reference the `Container` variant; the corrected scoped command
used `--features daytona,container` and passed. This feature-gating defect
predates the cut and is not a passed narrow-feature check.

T04/RM-PROVIDERS remain `in_progress`; A30 remains `not_run`. Other provider
creation paths, Daytona account/auth, image building, SSH gateway and cloud
saved-device features remain reachable. Native UI, Windows/Linux, live Core,
real provider endpoints and installed services were not exercised.
