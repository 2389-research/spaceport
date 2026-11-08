# Native platform results: development fork

This is the observed native desktop state, not a release qualification. Test
processes used isolated fixture paths. No signed package, personal daemon,
installed Core or real tailnet was changed.

| Platform | Native evidence | Open gate |
| --- | --- | --- |
| macOS | The pinned upstream and initial fork both launched fixture windows; `native_sidebar` failed on Orca's 79.47071 px glyph advance in a 79 px box, though the glyph was unclipped ([baseline](t00-baseline.md), [upstream log](t00-upstream-native-sidebar.log), [fork log](t00-native-sidebar.log)). A later fork run with a one-device-pixel tolerance passed 72 Classic frames, Superset, Orca, Minimal and dialog checks, then failed at decoy activation (`key=false`, `resized=true`, `app_active=false`, `app_hidden=false`) ([follow-on log](t03-native-sidebar-tolerance.log)). At `c3cfdf82`, the isolated `native_pages` fixture also launched but failed before page input because its window never became active ([page fixture](t11-native-pages-activation.md)). The diagnostic run found a visible regular-policy window while `loginwindow` held the foreground. | Run on an unlocked idle desktop, compare pinned upstream activation, then rerun retained Core workspace, chat, input, notification, package and updater scenarios in the final app. |
| Windows | No native Spaceport run. Source and cross-platform tests cannot prove the named pipe, launcher, input, notifications or package/update behavior. | A50 and package/identity scenarios not run. |
| Linux | No native Spaceport run under Wayland or X11. Source and cross-platform tests cannot prove socket, input, clipboard, notification or package behavior. | A51 and package/identity scenarios not run. |

The private pinned-Core macOS tests under T00/T02/T03/T05 exercise real Core
processes and local/fixture remote methods, but they do not open the final
Spaceport desktop UI. A49–A51 remain `not_run` as complete acceptance cases;
G-PLATFORM remains open. The current branch has ongoing source removals, so
these earlier native results do not qualify the later code revisions.
The [decoy activation audit](t11-sidebar-activation-audit.md) narrows the
earlier failure to an inactive AppKit test process and records the missing
upstream comparison. The newer page fixture found a visible, regular-policy
window while `loginwindow` was frontmost. This does not classify the cause or
close A49.
