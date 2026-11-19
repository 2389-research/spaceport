# Native error announcements

Date: 2026-11-19. Scope: `kjkw` under `9s44`.

## Qualification contract

Start only the selected test daemon and an owned GUI in the shared private sandbox.
Keep the status bar hidden and enable the existing QA menu in that sandbox's settings.
Subscribe to `LiveRegionChanged` on the exact GUI window through Windows UI Automation.
Use a bounded C# callback on a dedicated MTA thread.
Signal readiness only after registration succeeds.

Break only the sandbox's `config.local.toml`, then reload through the app's existing control command.
Require the visible warning and its named live-region event.
Restore the private configuration and require the warning to disappear.
Invoke the existing Custom toast preview and require its named live-region event.
Enter Zen mode and repeat the real configuration failure.
Require the warning to remain visible and announce in that layout.
Unregister on the registration thread and reap all exact owned children.

The Mac cannot execute this Windows-only check.
Local validation covers Rust types, lints, static checks, and review.
The native Windows result below qualifies event delivery on the published source.

## API evidence

Microsoft documents the [managed live-region event](https://learn.microsoft.com/en-us/dotnet/api/system.windows.automation.automationelementidentifiers.liveregionchangedevent).
Use that identifier and require event ID `20024`.
Follow Microsoft's [threading requirements](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-threading) and [subscription lifetime](https://learn.microsoft.com/en-us/dotnet/framework/ui-automation/subscribe-to-ui-automation-events).
Keep callbacks in C# because UIA invokes them on threads without a PowerShell runspace.
Microsoft's managed-client [property schema](https://github.com/dotnet/wpf/blob/main/src/Microsoft.DotNet.Wpf/src/UIAutomation/UIAutomationClient/MS/Internal/Automation/Schema.cs) does not establish reliable `LiveSetting` access.
The runner's exact Framework DLL remains uninspected.
This check measures native announcement delivery and names, rather than urgency values.

## Local evidence

The real parser accepts the shared QA and hidden-status configuration.
It rejects both malformed fixtures and preserves their distinct stage tokens in diagnostics.
That host check passes one test with no failures or ignored tests.
Each native event must contain its stage token, so a delayed earlier warning cannot qualify Zen.
Independent review approves the stage correction, window ownership, bounds, and cleanup.

Both host and Windows GNU lint configurations pass.
Format, file size, sorting, operations, localization, and diff checks pass.
Logs are `/tmp/spaceport-9s44-native-announcements-{parser,final-lint,static}.log`.
The host reports only the existing linker unwind and `block` future-compatibility notices.

The Windows run below qualifies C# compilation, native event delivery, and observer removal.
Existing warning and toast baseline regressions remain documented in [error feedback](review-error-feedback.md).

## Native receipt

Source `88566686c5a26ae59538ef1058bdf10c3b0b0418` passes the [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44491844895/job/135449440545).
All five UIA cases pass, with no failures or ignored cases.
The same job passes seven native GUI cases and the pipe-owner check.
The new announcement scenario compiles and executes the actual C# listener.

The log records `UIA_LIVE_READY` before receiving events.
The first warning event includes `Load GUI config:` and `native_hidden`.
The toast event names `Local: Custom notification. QA preview: a custom notification message.`
After Zen begins, the warning event includes `Load GUI config:` and `native_zen`.
The final `UIA_LIVE_REMOVED` receipt follows successful listener removal and thread exit.
The parent reaps the observer and its exact owned children.

The [complete CI run](https://github.com/2389-research/spaceport/actions/runs/44491844895) succeeds on this exact source.
All 18 executed jobs pass; one unconfigured target skips.
The separate supply-chain run also succeeds.
The Windows all-feature workspace job passes all six non-GUI harness cases, including timeout reaping.
Its five native cases remain opt-in there and execute in the separate native job above.
The real parser fixture test also passes in that Windows workspace job.

The log is `/tmp/spaceport-9s44-native-announcements-x64-green.log`.
The ARM native job also passes, but its matrix does not run the UIA scenario.
This receipt qualifies named native live-region delivery on Windows x64.
It does not measure urgency values or audible screen-reader speech.

Kata `kjkw` closes at `2026-11-19T09:09:30.504Z` with event `9787`.
The event UID is `01M7SH2XR8A213AJTE4J31JRJ5`.
Fresh issue and parent reads confirm the closure and 42 closed children with 11 open.
