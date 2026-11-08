# T04: remove the Herdr 1 install offer

At source baseline `6e61e119`, `window.rs` opened `Page::Install` after a
missing Herdr 1 executable and an unavailable Core connection. The page had
an Install button that opened the Herdr website, a keyboard shortcut, a QA
menu action, and locale strings. `endpoint/polling.rs` kept the page open
across retries. This was a Herdr 1 installation offer in a product whose
runtime must be a separately installed Herdr Core.

The source-absence test first failed on `ShowHerdrNotDetected`. The headless
window test first found the old `Install Herdr` instruction in the status bar.
This cut removes the page, action, keyboard, QA, retry exception, and locale
strings. The status bar reads the existing Core hub: while the local Herdr 1
executable is missing, Connecting and Unavailable both show `Herdr Core not
connected. Check the separate installation.` Ready shows `Herdr Core connected
(terminal fallback)`. A transient Core retry cannot make the message claim
that the fallback is connected. It does not claim workspace or chat parity.
The former install-dialog test now retains its separate reconnect and
background-retry dialog-fencing check.

Focused checks passed: headless Core status tests 3/3, retained retry-dialog
fencing 1/1, sidebar text-width 1/1, en-XA menu 2/2, Python source-absence
1/1, `just format-check`, `just check-file-size`, `just check-sorted`,
`just check-i18n` (5323 messages in 280 files), and `git diff --check`.
The GUI tests used an isolated Cargo target and emitted the existing macOS
linker `__eh_frame` warning and `block v0.1.6` future-compat notice.
No personal daemon, Core process, or user configuration was touched.

The Herdr 1 default bridge and snapshot consumers still run; removing that
runtime path needs the Core-first retained UI route described in
`t04-default-bridge-dependency.md`. T04/RM-LEGACY remain in progress and A32
is not yet verified. Native desktop, Windows/Linux, full workspace and release
checks were not run for this slice.
