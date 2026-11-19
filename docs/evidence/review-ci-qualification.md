# Draft review CI qualification

Scope: repair failures from [PR 125's first complete run](https://github.com/2389-research/spaceport/actions/runs/44421915465).
The tested public head was `6d89f5e42c0478aa865df329720f4a017e3764f5`.
This run failed. Earlier focused local results did not prove platform qualification.

## Repairs

Linux Clippy rejected an attribute gap and a redundant qualification.
Removed both without changing runtime behavior.
Linux verification remains assigned to hosted CI.

Windows native fixtures requested 480 pixels while ordinary windows enforced a 640-pixel minimum.
The actual window-options producer now exempts explicit isolated fixtures from that minimum.
Ordinary windows retain their 640-by-400 minimum.
The strict native width assertion remains unchanged and now reports the observed width.
The regression first failed with the ordinary minimum and then passed with the exemption.
Windows native qualification remains pending.

The ACP discovery fixture used Unix paths on Windows.
It now supplies native absolute paths through `join_paths`.
The production security filter remains unchanged.

Windows resources still embedded the inherited product name.
Both executables now read the canonical dependency-free product-name module.
The existing installer smoke checks both executables' description and product-name fields.
It retains shortcut targeting and AppUserModelID checks.
Real Windows metadata verification remains pending.

Global gesture feedback had lost its terminal-cell placement.
Terminal feedback now uses the actual padded cell container.
Other views retain window-body feedback.
The regression suite keeps padding and Devices/Zen visibility assertions.
Independent review caught an intermediate paint-order error.
The chat hover regression fails when chat paints over the warning.
Its first fixture attempt placed the warning outside chat and did not prove that regression.
The corrected fixture explicitly uses the top-left position inside the chat pane.

The retry-dialog fixture observed a retired handle before the reader finished draining.
It now waits for the production `retry_ready` fence before advancing the retry clock.
The dialog and generation assertions remain intact.

The notice-expiry fixture now starts its expiry interval after publishing the replacement.
This avoids measuring its reading time from before the replacement exists.
The CI failure did not reproduce locally; this timing explanation remains an inference.
The test still rejects inherited hover pauses.
The smoke error test now checks the typed source's current Display text.

## Isolated macOS runtime

Ten all-feature failures explicitly reported a missing Sparkle library before fixture entry.
Two additional child failures omitted stderr and had the same launch shape.
Their failure messages now include their private fixture logs.
Their deadlines and cleanup assertions remain intact.

The `stage-tests` command validates SDK bytes against the pinned archive.
It exposes that framework at both existing Cargo executable-relative lookup locations.
It refuses conflicting files and preserves them for diagnosis.
CI stages these links after restoring the Cargo cache and before all-feature tests.
Production linker paths and sandbox environment clearing remain unchanged.

For local all-feature tests, use the existing pinned SDK preparation first.
Then stage the framework in the selected Cargo target's debug profile:

```sh
python3 scripts/release/sparkle_framework.py stage-tests target/debug --sdk "$SPARKLE_FRAMEWORK_PATH"
```

If `CARGO_TARGET_DIR` is set, replace `target/debug` with that directory's `debug` path.
The command prints both prepared locations.
This prepares test dependencies; it does not qualify updates or release packaging.

## Verification

The native runtime regression failed before `stage-tests` existed.
Both runtime cases then passed against the real pinned SDK.
The test compiles real Objective-C executables and runs them with an empty environment.
It covers both Cargo executable layouts and repeated staging.
The conflict case verifies that an existing file survives refusal.

The final all-feature scopes pass 109 cases with ten explicit resource fixtures ignored directly.
Their parent tests invoke the applicable private fixtures.
The scopes include window options, padding, chat hit testing, Devices/Zen feedback, and the reported macOS failures.
All nine feedback cases pass after correcting paint order and fallback-group selection.
ACP path discovery passes two cases on macOS.
The existing Windows package script suite passes eight cases.
These macOS results do not prove Windows behavior.

Logs use `/tmp/spaceport-9s44-ci-repair-final-tests.log`, `ci-acp-native-paths.log`,
`ci-windows-package-unit.log`, `sparkle-stage-green.log`, and `flash-layer-verified.log`.
All short filenames share the `/tmp/spaceport-9s44-` prefix.

Formatting, size, sorted regions, operations, localization, and specification validators pass.
Both touched application crates pass all-target, all-feature Clippy with warnings denied on macOS.
The specification gate explicitly reports JSON Schema validation as not run.
Hosted platform checks, native Settings input, accessibility, and release profiling remain pending.
No Kata issue closes solely because these local checks pass.
