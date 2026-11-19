# OS accessibility preferences

Verified on macOS with Rust 1.99.0 on 2026-11-17.
`krqj` is implemented and independently reviewed.
The OS preference portion of `8zvq` is implemented.
Its separate secondary-text ratio correction remains open.

## Ownership and behavior

`accessibility::policy` owns one atomic snapshot for UI and background theme readers.
Only startup and the UI watcher update the production snapshot.
One apply function updates GPUI and manual motion together.
Scoped test overrides use the same apply/read functions without changing the production snapshot.

macOS reads safe `NSWorkspace` display getters before startup palettes load.
Windows uses safe `UISettings` and `AccessibilitySettings` getters.
The obsolete Windows FFI contrast reader was removed.
Linux reads the desktop Settings portal asynchronously on the background executor.
Its two-second deadline includes connection, authentication, and the method reply.
The app polls every two seconds after each completed read.
Unavailable keys preserve the last known preference; initial unknown values keep normal behavior.
The watcher task belongs to the app and cancels when dropped.

Manual transitions finish immediately under reduced motion.
They stop requesting frames through the existing progress and Toggle consumers.
Pinned GPUI also finishes one-shot animations and freezes repeating animations under its flag.
Its animation implementation schedules no further frames in that state.

OS high contrast raises the configured contrast floor.
Raw palettes remain authoritative for asynchronous reloads and draft caches.
The current policy applies when the UI consumes those palettes.
Active app drafts retain their prepared raw colors.
Unapplied runtime-only drafts do not block app color reloads.
Picker refresh preserves accepted-save tokens and rejects stale preview sessions.
Cancel and accepted-save completion reload stale contrast without losing the selected theme.
A Logs-only window refreshes its existing appearance without reading diagnostic records.
Terminal palette colors remain unchanged.

## Regression and checks

The initial manual-motion regression failed with `Some(0.0)` instead of a completed transition.
The real D-Bus peer rejected a fixed Rust array encoded as `(s)` instead of `as`.
The final client passes a slice and receives the real typed `ReadAll` reply.
Corrected app-context regressions reproduced stale picker and Logs-only colors.
The runtime-only draft regression then reproduced an overly broad app-color ownership guard.
All final regressions pass.

Cargo calls held the task's shared lock and used its shared target directory.
They set `CARGO_INCREMENTAL=0` and `RUSTUP_TOOLCHAIN=1.99.0`.
The final batch compiled once with:

```sh
cargo test -p sheprd-gpui --bin spaceport --no-run --message-format=json --locked
```

A throwaway runner used `crates/test-support/sandbox.rs` for every test process.
It cleared inherited environment and isolated HOME, config, and socket paths.
It waited for and reaped only its exact children before removing fixtures.
The compiled test binary received each filter below:

| Filter | Passed |
| --- | ---: |
| `accessibility::policy` | 6 |
| `motion::` | 12 |
| `settings_window::` | 250 |
| `theme_picker::` | 17 |
| `log_window::` | 16 |
| `sidebar::tests::working_marks` | 2 |
| `settings_panel::` | 14 |
| `contrast::` | 15 |

The policy tests include a read-only native AppKit probe and an isolated real D-Bus exchange.
The rendered status-mark consumer verifies that reduced motion stops frame requests.
The Settings tests cover app drafts and unapplied runtime drafts without saving either.

```sh
cargo clippy -p sheprd-gpui --bin spaceport --tests --locked -- -D warnings
```

That Clippy check passed.
Scoped Rust formatting, `git diff --check`, and `bash scripts/check-file-size.sh` passed.
Root independently reviewed ownership, deadline cancellation, startup, palette consumers, and picker/save fences without remaining findings.
Existing linker compact-unwind and `block` future-incompatibility notices remain unchanged.

The exact batch is `/tmp/spaceport-qnz9/check-accessibility.sh`.
Its summary is `krqj-final.log`; child logs are under `accessibility-consumers/` in that directory.
Regression logs are `krqj-red.log`, `krqj-wire-signature-red.log`, `krqj-picker-red.log`, and `krqj-draft-red.log`.

## Primary API references and limits

Apple documents [Reduce Motion](https://developer.apple.com/documentation/appkit/nsworkspace/accessibilitydisplayshouldreducemotion)
and [Increase Contrast](https://developer.apple.com/documentation/appkit/nsworkspace/accessibilitydisplayshouldincreasecontrast).
The pinned `objc2-app-kit` 0.3.2 bindings expose both as safe getters.
The native probe compares their actual values without changing OS preferences.

Microsoft documents [AnimationsEnabled](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.uisettings.animationsenabled?view=winrt-28000)
and [HighContrast](https://learn.microsoft.com/en-us/uwp/api/windows.ui.viewmanagement.accessibilitysettings.highcontrast?view=winrt-28000).
The pinned `windows` 0.62.2 bindings expose safe getters.
At the original checkpoint, Windows runtime qualification remained pending.
Cross-compilation is not native Windows evidence.

The [Settings portal specification](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html)
defines `ReadAll(as)`, `contrast`, and `reduced-motion` under `org.freedesktop.appearance`.
Both preferences use unsigned integers: one requests the feature; zero and unknown values mean no preference.
Missing or wrongly typed keys remain unknown in the app.
The Unix wire test proves serialization and decoding, not a Linux desktop backend's support.
Linux native qualification remains pending and requires a backend that provides these preferences.
The initial Linux lookup is asynchronous and can leave the first frame at normal policy.

Native screenshot ratios, keyboard checks, and performance checks remain pending.
No OS preference, personal configuration, runtime, or daemon was changed by these checks.

## Reorder extraction follow-up

The later reorder extraction left three manual slide consumers outside the reduced-motion policy.
Workspace, device, and tab slides passed their fixed duration directly to the shared geometry crate.
Unit tests settled those slides immediately, hiding the normal-build regression.
Opted-in chat glide also kept smoothing new output and active motion after the policy changed.

One client-side duration helper now applies reduced motion before all three slide consumers.
Each consumer reuses that duration for retargeting, position, and frame-request decisions.
The shared geometry crate retains its normal 150-millisecond duration.
Chat uses its existing immediate-follow branch under reduced motion.
Pending user scrolls and selected text retain their existing precedence.

Three actual regressions fail before the correction.
The real-duration slide remains at 8.75 instead of 10.
New chat output remains 142 pixels behind the end.
Enabling reduced motion leaves an active chat glide unfinished.
The corrected motion, reorder, chat-glide, and working-mark scopes pass 70 cases, with none ignored.
The separate normal-consumer timing test also passes.
Independent review finds no remaining material issue in these consumers.

Published source `46738c99c` already passes the native Windows getter test on x64 and ARM.
It also passes the actual AppKit getter test on macOS.
The Linux job passes the real portal-wire, policy, transition, and working-mark tests.
Verified [Windows x64](https://github.com/2389-research/spaceport/actions/runs/44452942264/job/135321241319),
[Windows ARM](https://github.com/2389-research/spaceport/actions/runs/44452942264/job/135321241301),
[macOS](https://github.com/2389-research/spaceport/actions/runs/44452942264/job/135321241330),
and [Linux](https://github.com/2389-research/spaceport/actions/runs/44452942264/job/135321241351) logs.
Those results qualify the unchanged OS readers and earlier consumers, not the new local corrections.
The Linux desktop-backend limitation above remains explicit.

Logs: `/tmp/spaceport-9s44-reduced-motion-consumers-{red,green}.log` and `reduced-motion-normal-consumer.log`.
Short names share the `/tmp/spaceport-9s44-` prefix.
Kata `krqj` closes after publication of fix `1ff2bb922` and final scoped verification.
Its closure event is `9782` (`01M7S1ZEDMQ46YPVEYN7RZBY6H`).
A fresh issue read confirms its closed status.
This closure does not qualify the separate UI Automation timeout, speech, Settings input, or release performance findings.
Canonical host and Windows GNU lint pass both feature configurations.
The Windows GNU checks pass again after adding the timeout diagnostic.
Formatting, size, sorting, operations, localization, specification, and diff validators pass.
The specification gate explicitly skips JSON Schema validation.
Logs use `/tmp/spaceport-9s44-reduced-motion-{host-lint,windows-lint,final-static}.log`.
The diagnostic cross-check log is `/tmp/spaceport-9s44-uia-diagnostic-windows-lint.log`.
