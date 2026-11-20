# Application menu departure

The custom application popup shares Spaceport's reversible popup motion.
The native macOS menu bar remains under the operating system.

## Contract

Closing captures display rows, selection, labels, geometry, and the numeric scroll offset.
The captured data retains no action tree, focus handle, or scroll handle.
The active owner resets before departure paints.
Configuration invalidation captures the same data before releasing the menu.

Active and outgoing panels share their row and back-button templates.
Outgoing panels register no pointer, keyboard, scroll, or action callbacks.
They expose no menu or item accessibility roles.
They never reinstall the Windows native-page cover or the duplicate application bar.
Their bounds stay within the current viewport after a resize.

Live rows remain direct scroll children.
This preserves the child indices used by keyboard selection and reveal.
Only outgoing rows use a wrapper with the captured numeric offset.

Held F10, Enter, Space, and Escape cannot activate or dismiss the popup.
Held navigation keys still move selection.

## Regression evidence

The actual departure baseline lost the panel after its active owner reset.
See `/tmp/spaceport-9s44-application-runtime-red.log`.
The actual held-F10 baseline closed the active popup.
See `/tmp/spaceport-9s44-application-repeat-red.log`.

The final focused checks pass in the immutable executable
`/tmp/spaceport-qnz9/ydr4-workspace-green-tests`.
The default build completed with Rust 1.99.0 and the locked dependencies.

Commands:

```sh
/tmp/spaceport-qnz9/ydr4-workspace-green-tests menus::in_window::tests --test-threads=1
/tmp/spaceport-qnz9/ydr4-workspace-green-tests menu::appear::tests::application --test-threads=1
```

All eleven application-menu consumers pass.
Both actual shared-departure tests pass.
Logs: `/tmp/spaceport-9s44-application-consumers-green.log` and
`/tmp/spaceport-9s44-application-departure-green.log`.

The checks cover config invalidation, action dispatch, disabled actions, checked submenus, and focus restoration.
They also cover real scroll positions, End reveal, resizing, held keys, and detached stale controls.
The detached renderer has no parent input shield.

Independent review approved action, focus, scroll, and Windows-cover ownership after these checks passed.
Whole GPUI lint after the remaining popup families remains pending.

Native Linux and Windows input, focus, drawing, and accessibility checks remain pending.
No headless test proves native platform behavior.
