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
Native delivery remains pending the actual Windows CI result.

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

The C# compiler, native event delivery, and observer removal receipt still need the Windows run.
Existing warning and toast baseline regressions remain documented in [error feedback](review-error-feedback.md).
Kata `kjkw` remains open until native evidence qualifies this boundary.
