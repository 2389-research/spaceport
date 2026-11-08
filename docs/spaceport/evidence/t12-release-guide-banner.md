# Inherited release guide warning

The inherited `scripts/release/README.md` still describes Sheprd package names,
release URLs and update ownership. A new banner at its start identifies it as
inherited source documentation and points to the current Spaceport release
identity audit and gap record. The release build gate still rejects
`HERDR_RELEASE_VERSION`; this banner does not change any build, install or
publish route.

The prior release audit's user-facing-doc row predates this banner and remains
historical evidence. Its statement about the detailed Sheprd instructions is
still true. The root README was corrected in a separate signed slice.

The two new relative links resolve, and `git diff --check` passed. No package,
installer, updater, native app, or personal environment was exercised. T12 is
blocked by G-RELEASE and G-PLATFORM; A42–A44 stay not_run.
