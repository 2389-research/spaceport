# g9jb — external browser opens listening ports (evidence)

Recorded 2026-11-21 21:01 CST by the spaceport-factory session.

## Build and host

- Build: `d2e1cdd66` (debug, `cargo build --locked -p sheprd-gpui --bin spaceport`)
- Host: macOS 26.7.1, default browser Safari (`com.apple.safari` in LaunchServices)
- Runtime: the live Herdr 1 daemon (session `default`) with the Spaceport window
  on workspace `wN` (local host).

## Procedure

1. A listener was started **inside a Herdr pane** (the spaceport-factory pane,
   `wN:p1`), so it inherited `HERDR_WORKSPACE_ID=wN`:

   ```sh
   python3 -m http.server 8765 --bind 127.0.0.1
   ```

2. The app's scan saw it within seconds:

   ```
   $ spaceport ports
   Workspace wN on Local listens on:
     localhost:8765           python3.12  http://localhost:8765/
   ```

   The scan's attribution rule is real: listeners without the workspace
   environment variable are dropped (the `listening_ports/scan.rs` parse).
   A server started outside a Herdr pane produces no chip at all.

3. The open route was driven through the CLI twin, which shares
   `open_workspace_page` with the chip's click handler:

   ```
   $ spaceport ports open 8765
   Opened port 8765 in workspace wN
   ```

4. Safari launched in response. Process evidence at the time of the call:
   `Safari` pid 95215 with fresh `WebKit.WebContent` (89565, 89622) and
   `WebKit.GPU` (89431) XPC children stamped 21:01:01–21:01:03 — seconds
   after the `ports open` call. The page itself could not be read back
   (Apple events to Safari are not authorized for this session), so the
   URL inside Safari is attested by the timing correlation and the
   `PageOpened::SystemBrowser` response, not by reading the browser.

## What this closes, and what it does not

- Proven live: scan → attribution → chip data → `open_workspace_page` →
  `cx.open_url` → the OS opened the default browser.
- The remaining hop — the chip's own `on_click` calling `open_port_link` —
  has no live recording. It is covered by the equivalent headless click test
  (`group_menu/tests/new_tabs.rs` clicks the menu row through the same
  `open_port_link`), and the static trace
  (`listening_ports/render.rs:145-151`) shows the chip wired to the same
  method. A manual chip click was offered and not performed; the CLI route
  recorded here exercises every shared call.

## Explanation of the original report

The one-line report ("we removed the browser, but we should allow for an
external browser to open listening ports") matches the pre-#142 world. Since
#142 the embedded webview route is gone and every port link opens the system
browser through `open_workspace_page` (see `t04-browser-system-open.md`;
`browser/view.rs` has no webview branch). The repro above shows that working
on this build. The likely reason a reporter saw "no browser" is §1.4 of the
factory's static trace: a listener started outside a Herdr pane is invisible
to the scan, so there is nothing to click.
