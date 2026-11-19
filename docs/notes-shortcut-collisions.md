# Default shortcut collisions (review finding 3wd4)

The finding: (a) the Linux Ctrl-Shift digit and punctuation defaults (tab
1–9, `[`, `]`, `,`, `\`, `` ` ``) likely never fire, because Linux delivers a
Shifted key as its symbol and only the font keys had symbol aliases; (b)
`ctrl-alt-n`, `ctrl-alt-b`, `ctrl-alt-z`, and `ctrl-alt-w` collide with AltGr
letters on Windows layouts, where AltGr is Ctrl-Alt; (c) bare F6, Shift-F6,
F11, and Ctrl-PageUp/PageDown are taken by htop, mc, and programs in a pane;
(d) `ctrl-alt-shift` with the arrows or `r` may be a GNOME key (unverified).

What was confirmed in the dependencies before changing anything:

- `gpui-pre-linux`'s keysym map translates `XF86`/`Next`/`bracketleft`… and
  clears `shift` whenever the key it reports is not case-varying. A physical
  Ctrl-Shift-1 therefore arrives as `{control}` + `!`, Ctrl-Shift-`[` as
  `{control}` + `{`, and Ctrl-Shift-, as `{control}` + `<`. All Shift-clearing
  punctuation defaults were dead on Linux.
- `gpui-pre-windows`'s `need_to_convert_to_shifted_key` covers `VK_0`–`VK_9`
  and the OEM punctuation keys and clears `shift` the same way, so the Windows
  profile needs the same symbol aliases (it inherits them from the Linux
  profile). AltGr there arrives as Ctrl-Alt with `prefer_character_input`, so a
  binding on Ctrl-Alt and a letter takes the letter an AltGr layout types.
- macOS is untouched: Cmd bindings already are the platform-verified ones.

Every change below is on the Linux column of the catalog (inherited by
Windows) or in `shortcuts/windows.rs`; the macOS column is unchanged.

## (a) A shifted digit or punctuation key arrives as its symbol

| Command | Before (Linux, Windows) | After |
| --- | --- | --- |
| `focus_tab_1` … `focus_tab_9` | `ctrl-shift-1` … `ctrl-shift-9` | `ctrl-shift-1 ctrl-!` … `ctrl-shift-9 ctrl-(` |
| `back` | `ctrl-shift-[` | `ctrl-shift-[ ctrl-{` |
| `forward` | `ctrl-shift-]` | `ctrl-shift-] ctrl-}` |
| `previous_pane` | `ctrl-alt-shift-[` | `ctrl-alt-shift-[ ctrl-alt-{` |
| `next_pane` | `ctrl-alt-shift-]` | `ctrl-alt-shift-] ctrl-alt-}` |
| `settings` | `ctrl-shift-,` | `ctrl-shift-, ctrl-<` |
| `keybindings` | `ctrl-shift-/` | `ctrl-shift-/ ctrl-?` |
| `split_editor` | `ctrl-shift-\` | `ctrl-shift-\ ctrl-|` |
| `cycle_windows` | `ctrl-shift-`` | `` ctrl-shift-` ctrl-~ `` |

`increase_font_size`, `decrease_font_size`, and `reset_font_size` already
carried the alias form (`ctrl-+`, `ctrl-_`, `ctrl-)`), so this follows the
pattern that was already in the table rather than inventing one. The digit row
declares the alias once more per number: a numbered row now has a `symbols:`
column, and `catalog!`'s `@symbols` step maps a digit to the symbol it types.
The row still names each command once. `#![recursion_limit]` rose from 512 to
1024 for the extra expansion.

A number is now captured as `tt` rather than `literal` in the numbered row
arm: a captured `literal` is an opaque token and no longer matches the specific
`1`…`9` of each `@symbols` arm, which made the macro recurse until the limit.

## (b) AltGr is Ctrl-Alt on Windows

| Command | Before | After |
| --- | --- | --- |
| `new_window` | `ctrl-alt-n` both hosts | `ctrl-shift-n` on Linux and Windows |
| `toggle_zen_mode` | `ctrl-alt-z` both hosts | `ctrl-shift-z` on Linux and Windows |
| `toggle_status_bar` | `ctrl-alt-b` both hosts | Linux `ctrl-alt-b`; Windows `ctrl-alt-shift-b` |
| `close_window` | `ctrl-alt-w` both hosts | Linux `ctrl-alt-w`; Windows none (Alt+F4) |

- New window is Ctrl-Shift-N, as Windows Terminal opens one, and Zen mode keeps
  its `z`. Both drop Ctrl-Alt entirely, so the `ń` a Polish layout types with
  AltGr-N and the `ż` of AltGr-Z reach the pane.
- The status bar keeps the sidebar's `b` on the app's second tier. AltGr-Shift-B
  types nothing on the layouts that put letters on AltGr (Polish, Brazilian,
  Vietnamese), where plain AltGr-B would type a letter.
- Closing a window has no chord left to move to: `ctrl-alt-shift-w` is Close
  Tab, and Windows closes a window with Alt+F4 through the system, which the
  app documents and its keymap leaves free. `Ctrl-Alt-W` on a Brazilian ABNT2
  layout types `?`, so it is gone rather than kept.
- Linux keeps its `ctrl-alt-*` chords because X11 and Wayland report AltGr as
  level 3 (`Mod5`), not as Alt, so those chords do not type anything there.
  The reasoning lives in `shortcuts/windows.rs`, whose whole job is Windows
  divergence, and in the catalog rows.

## (c) Keys the programs in a pane already claim

| Command | Before | After |
| --- | --- | --- |
| `focus_next_region` | `f6` | `ctrl-f6` |
| `focus_previous_region` | `shift-f6` | `ctrl-shift-f6` |
| `toggle_full_screen` | `f11` | `ctrl-shift-f11` |
| `next_tab` | `ctrl-pagedown` | `ctrl-shift-pagedown` |
| `previous_tab` | `ctrl-pageup` | `ctrl-shift-pageup` |

mc moves with F6 and renames with Shift-F6, and htop sorts with F6, so bare
F6 was the app's only default that a program in a pane needed. Keeping the F6
name under the app's own Ctrl prefix mirrors macOS (`cmd-f6` / `cmd-shift-f6`)
and stays out of the pane programs' way.

Full screen keeps the F11 name with the same prefix: the platform's full-screen
key is a function key a pane program can also want, and the app already owns
Ctrl-Shift. Ctrl-PageUp and Ctrl-PageDown page a pane program's own output, so
tab cycling takes the same tier (`ctrl-tab` was rejected: the palette uses
Ctrl-Tab and Ctrl-Shift-Tab to cycle its filters).

`tests/accessibility_windows.rs` now drives `^{F6}` and `^+{F6}`, and the
README's chrome section names Ctrl+F6.

## (d) Ctrl-Alt-Shift arrows and Ctrl-Alt-Shift-R

`focus_left/right/up/down` (`ctrl-alt-shift-arrows`), `resume_palette`
(`ctrl-alt-shift-r`), and the rest of the Ctrl-Alt-Shift tier are unchanged. No
layout or desktop was measured here, and the finding itself says the GNOME
question needs a real keyboard. The rows stay as they are; the catalog comment
above the table records that AltGr-Shift types the capital of an AltGr letter
where a layout has one, which is the same unverified class.

## Not fixed, and worth a follow-up

`shortcuts/windows.rs` keeps the pre-existing overrides that give Windows only
`ctrl-shift-=` / `ctrl-shift-+` and `ctrl-shift--` for the font rows, with the
comment "Linux's extra symbol aliases match its backend's Shift removal".
`gpui-pre-windows`'s `get_keystroke_key` clears Shift for `VK_OEM_PLUS` and
`VK_OEM_MINUS` too, so those two Windows defaults look like the same dead
punctuation class as (a): the delivered keystroke is `{control}` + `+` and the
bound keystrokes both carry Shift. This change leaves them alone because the
finding did not name them and no Windows host was available to confirm, and it
notes the class here instead.

## Verification

- `cargo test -p sheprd-commands --locked` — 40 pass, including the new
  `shortcuts::tests::collisions` regressions, the repinned
  `tests/fixtures/default_shortcuts.txt`, and the profile invariants
  (`linux_profile_…`, `windows_profile_…`) that run on every host.
- `cargo test -p sheprd-gpui --bin spaceport --locked -- keymap::` — 57 pass.
- `cargo fmt --all -- --check`, `just check-file-size`, `just check-sorted`,
  `just check-commits`.
- Not run: the `#[cfg(linux)]` and `#[cfg(windows)]` keymap tests (this run was
  macOS), `tests/accessibility_windows.rs` (needs a Windows desktop and a
  daemon binary), and any native keyboard check. No real keyboard verified the
  AltGr or GNOME claims.
