# T04/RM-MODS: legacy plugin command routes

At `2a0e3c22`, Herdr 1 `ClientShellSnapshot.commands` still fed plugin actions into the palette, direct and prefix key matching, Settings > Keyboard Shortcuts, and the local `keybinds` control reply. A palette or key activation sent `command.invoke`; plugin actions also sent the selected pane's cell range. The existing `ClientShellCommandAction` wire enum remains because the transitional Herdr 1 snapshot decoder still reads it. Shell, pane and popup custom commands share that wire path and remain reachable until the Core-first default window can replace them.

The app now filters `PluginAction` and unknown actions at one `keymap::available_commands` seam used by the palette, Settings reference and control reply. Key matching uses the same filter; palette invocation refuses a stale plugin action even if one is supplied after the menu opened. The plugin-only selection capture, revision fence and tests were removed. Ordinary selection/copy logic and tests remain; the unused plugin-specific selection range helpers were removed. The ordinary Sidebar config parser, Integrations and agent skills were not touched.

The first focused test run failed on all three new assertions: plugin actions appeared in the palette, could be invoked from a captured target, and took a direct shortcut before a shell command. After the change, those three passed. The final focused plugin-action run passed 4/4 (palette visibility, stale invocation, direct/prefix shortcut and Settings visibility); retained custom shortcut UI passed 5/5; connected-only custom key behavior passed 1/1; terminal selection tests passed 24/24. Default app production check and all-feature app production check passed without new code warnings. The test linker reported its existing macOS compact-unwind warning, and Cargo reported the existing `block v0.1.6` future-compatibility notice. `just format-check`, `just check-file-size`, `just check-sorted` and `just check-i18n` passed. Package-root `spec.py check` passed schema and link validation, and `test_spec.py` passed 21/21. The first direct Python invocation failed only because `jsonschema` was missing from that interpreter; the final package commands used `uv run --with jsonschema==4.26.0`.

This does not finish RM-MODS or A30. The transitional Herdr 1 bridge and non-plugin dynamic custom command path remain. Plugin link handlers and host-provided agent view labels are separate reachable routes. Native GUI, Linux, Windows and full workspace CI were not run for this slice.

## Merged branch verification

Cherry-pick `7016ac25` kept the concurrent Daytona and checkpoint evidence
links. On the combined branch, stale/hidden palette tests passed 2/2, direct
key filtering 1/1, Settings/CLI visibility 1/1, retained custom shortcut UI
5/5 and terminal selection 24/24. Default and all-feature production app
checks, format, file-size, sorted and i18n checks passed (5,066 messages in
266 files). The package check and its 21 tooling tests passed. These checks
do not prove native interaction or absence of the legacy plugin link route;
A30 remains `not_run`.
