# Native sidebar routing without an unlocked desktop

Doctor Biz requested automated checks without repeated desktop unlocks.
The isolated AppKit fixture now has a separate routing test.
It shares the original native rendering and event-delivery code.
It needs a macOS WindowServer session, but no key-window assertion.
It does not bypass the lock, change its policy, or touch personal daemons.

Run from the repository root:

```sh
RUSTUP_TOOLCHAIN=1.99.0 just test-sidebar-routing
```

The macOS all-features CI leg also runs this test.
That step uses `integration-test`, matching the local recipe.
The preceding full workspace step still tests all features.
The original `just test-sidebar` keeps its key-window requirement and negative tests.
Neither command silently downgrades the other's success criteria.
Application startup may request activation; the routing test does not require its success.
The focused fixture initializes its own comfortable layout and scrolled header.
Its content height is 600 pixels, fitting the CI desktop.
It uses the shared host-interaction, resize, font, and independent-scroll checks.
It does not run the separate Settings, About, menu, or full-frame qualification.

## Regression and cause

The host-routing failure came from the fixture's click coordinates.
After scrolling, Local has a clipped original header and a visible pinned copy.
Text-keyed paint probes retain the first copy.
Its label started at y=82 with a mask starting at y=94.
The visible pinned label and endpoint control started at y=102.
The test delivered y=90 correctly, outside the visible header.
The real endpoint-selection handler was never called.

Header clicks now use their endpoint's painted control for the row's y coordinate.
That control records the visible pinned copy last.
Native event delivery, endpoint-selection code, and the key-window assertions remain intact.
Temporary production callback logging was removed.

## Evidence

Logs live in `.worktrees/native-host-selection/target/native-host-selection/`.

- `routing-shared-red.log`: locked-desktop test fails at step 3 before the coordinate fix.
- `routing-final.log`: native routing passes after the fix and final paint-error check.
- `routing-reviewed.log`: final renamed test and preserved original recipe coverage.
- `sidebar-final.log`: separate key-window qualification fails at activation while locked.
- `negative-fixtures.log`: both existing paint/preferences failure cases pass.
- `routing-focused-red.log`: the focused 600-pixel fixture fails at step 3 without the coordinate fix.
- `routing-focused-final.log`: the restored fix passes the focused fixture.

The passing routing run verifies all ten interaction steps.
Those include disabled-host refusal, duplicate-ID routing, repository scope, and unchanged decoy state.
It also verifies five resize allocations, three themed font sizes, and independent scrolling.
The earlier shared-driver runs additionally verified 72 native glyph frames.
Those frames remain part of the original desktop suite.
They are not part of the focused routing test.
Real AppKit events target only retained fixture views.
The existing private sandbox reaps only its own children.

The failed key-window run remains failed.
Routine routing qualification no longer needs that check to pass first.
OS key-window behavior, real IME interaction, full desktop acceptance, and backend gaps remain separate gates.
T11, A49, and broad macOS qualification remain open.
No product UI or runtime lifecycle behavior changed.

Quick format, file-size, sorting, Fluent, workflow, and whitespace checks pass.
Specification rendering and integrity checks pass; those do not establish application acceptance.
Independent review found and corrected a test-filter coverage loss before commit.
The original recipe still runs its two negative cases.
Existing Darwin compact-unwind and `block 0.1.6` warnings remain.
Full workspace and cross-platform checks are deferred to PR CI.

## CI feature selection

The initial CI step used `--all-features` for the native child.
Job `131439018421` failed before application startup.
The dynamic loader could not find `Sparkle.framework` and aborted the child.
The sandbox correctly clears inherited loader overrides.
An updater-enabled loose executable needs separate bundle qualification.

The native routing step now uses the existing `integration-test` feature selection.
It does not need the optional updater.
Sandbox isolation and all-features workspace coverage remain unchanged.
The failed CI result remains recorded; the corrected step requires a new passing run.

The corrected-feature run then reached an unrelated Settings check.
Job `131446854001` could not allocate the required 960-by-780 Settings viewport.
That desktop suite requires a larger display than the hosted runner provides.
The focused routing test now prepares its own state before the desktop suite starts.
It retains all ten host interactions and the shared resize/font/scroll assertions.
Large-font cases scroll their host into the compact viewport before inspecting glyphs.
Original Settings sizes, focus assertions, and desktop-suite coverage remain unchanged.
