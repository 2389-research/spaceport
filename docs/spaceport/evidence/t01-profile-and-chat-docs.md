# Profile removal and chat documentation

Doctor Biz approved the proposed `rxkr` documentation slice and `tj5q` profile removal.
The instruction was: "let's do rxkr and tj5q".
Decision D67 supersedes D50's profile retention only.
The source base is `c1e76c76a62c88d7fdaf20b2b9e0237f26863d54`.
The task branch is `fix/profile-and-install-docs`.

## Changes

The titlebar no longer draws an avatar or reserves its 40-pixel slot.
The profile popup, page variant, localized labels and dedicated routes are removed.
The shared application menu retains Settings, keyboard shortcuts and About.
It now also exposes the existing Themes command.
Native and in-window menus use the same definitions.
The status bar and command palette retain their local controls.

Runtime machine profiles and chat user icons are separate retained features.
No daemon, agent, credential or settings lifecycle changes.
No installed app or personal runtime was used as a fixture.

The CLI guide installs the helper from `2389-research/spaceport`.
The rich-chat and helper guides name Spaceport as the desktop client.
The helper guide no longer promises the removed phone chat list.
The `sheprd-acp` command and external protocol identifiers remain unchanged.
Legal attribution and the explicitly historical upstream guide remain intact.
The larger `rxkr` naming task remains open.

## Verification

Logs live in `.worktrees/profile-and-docs/target/profile-and-docs/`.
Cargo uses Rust 1.99.0, `--locked`, `integration-test`, and incremental compilation off.
This task reuses the compiler cache from `spaceport-display-copy`.
It does not modify the frozen Sparkle target or fixture.

- `red-verified.log`: avatar absence and missing Themes fail on the old implementation.
- The shared-action preservation test passes on that same baseline.
- `titlebar.log`: 32 header, navigation, status and tab-strip tests pass.
- `menu.log`: 305 popup and related menu tests pass.
- `menus-final.log`: 28 shared-menu and in-window menu tests pass.
- `themes.log`: 15 theme preview, saving and cancellation tests pass.
- `sidebar.log`: the retained sidebar and popup layout check passes.
- `focus.log`: three chrome focus-region tests pass.
- `native-before-corrected.log` and `native-after.log`: both native fixture runs pass.
- Each native run covers light/dark themes at widths 800 and 360.
- `clippy.log`: strict integration-feature, all-target app Clippy passes.
- Quick formatting, file-size, sorting and Fluent checks pass.
- Offline Cargo metadata confirms the actual `sheprd-acp` package and binary names.
- Independent read-only review reports no actionable finding.

Headless action dispatch opens shortcuts, themes, About and a real Settings view.
Native checks render the first three pages through the shared command handler.
They verify blocked terminal actions, Escape dismissal and restored popup focus.
Native About checks preserve visible Spaceport and upstream attribution glyphs.
The harness clears inherited environment and uses isolated state and exact-child cleanup.
It launches no daemon or agent and does not access the clipboard.

GPUI defers `Window::dispatch_action` until the enclosing update finishes.
The first native fixture incorrectly inspected its result in that same update.
The corrected rendering fixture calls the shared synchronous handler.
Headless tests separately exercise the actual deferred dispatch path.
No application dispatch semantics changed to accommodate this fixture.

## Native screenshots

Images show actual native window content, without OS titlebar pixels.
Before captures use the same fixture before the profile removal.
The captured workspace names and blank terminal are synthetic fixture data.

| Theme and width | Before | After |
| --- | --- | --- |
| Dark, wide | [Before](profile-removal/before-dark-800.png) | [After](profile-removal/after-dark-800.png) |
| Dark, narrow | [Before](profile-removal/before-dark-360.png) | [After](profile-removal/after-dark-360.png) |
| Light, wide | [Before](profile-removal/before-light-800.png) | [After](profile-removal/after-light-800.png) |
| Light, narrow | [Before](profile-removal/before-light-360.png) | [After](profile-removal/after-light-360.png) |

## Remaining checks

Full workspace and default/all-feature platform checks remain deferred to CI.
Linux and Windows native behavior was not checked on this Mac.
Windows checks remain paused under D66.
Native Settings window qualification is separate from this fixture's headless Settings check.
Existing Darwin linker and `block 0.1.6` warnings remain.
No helper was installed, and no release or tag was created.
The separate listening-port Kata `g9jb` remains open.
Broad T01, T04, T11 and T12 acceptance remains open.
