# Shortcut reference departure

Finding: `ydr4`. Verified partial work on 2026-11-17.
The complete popup finding remains open.

## Ownership and behavior

The shortcut reference uses one grouped row producer and one active/static renderer.
Its paint snapshot holds filtered rows, shared text, counts, input paint, and a plain scroll offset.
Static command metadata supplies the existing Edit labels.
The snapshot holds no entity, subscription, scroll handle, runtime target, or save task.

Closing still clears the active search subscription, search field, and idle editor.
The next frame uses input-free paint; its old field owners can then release.
Departing content retains the rows and editor display that were actually shown.
New daemon commands and replacement scroll owners cannot change that content.
Static Close, Edit, Save, Unbind, Reset, and Cancel controls register no action handlers.
The existing accepted-save operation and exact field reconciliation remain unchanged.
Accepted writes can retain their original field until completion; outgoing paint does not own that field.

The renderer moved from `menu/settings.rs` into `menu/settings/keybinds.rs`.
The command grouping table remains declared once.
Active and frozen input use the same field styling and shortcut editor template.

## Verification

Two actual window regressions failed before implementation.
Filtered rows followed a replacement command snapshot while closing.
The idle shortcut editor vanished after its active owner reset.
Red log: `/tmp/spaceport-9s44-keybinds-departure-red.log`.

Three departure checks now pass, including actual scrolling and identical text/body bounds.
The scroll check replaces the inactive scroll owner after capture.
The outgoing content retains its recorded offset.
A separate no-parent-shield test clicks old Close, Edit, Cancel, and Save controls.
Each click preserves the replacement editor's field identity, draft, and page.
Its replacement draft is invalid, so even a broken save guard cannot write personal settings.

All 15 shortcut/config consumers and eight editor consumers pass.
The current 31-case popup-motion scope also passes.
Logs:

- `/tmp/spaceport-9s44-keybinds-settings-final.log`
- `/tmp/spaceport-9s44-keybinds-editors-final.log`
- `/tmp/spaceport-9s44-keybinds-motion-final.log`
- `/tmp/spaceport-9s44-keybinds-controls-check2.log`

The first extra scroll fixture selected an offscreen late catalog row.
It now selects the verified first workspace row and proves real scrolling before comparing departure.
The Cancel control also gained its missing test selector.
Neither failed fixture contributes a passing result.

Independent review approved the capture types, shared catalog, action guards, and unchanged save reconciliation.
The source-stable aggregate checkpoint passes host and Windows GNU all-feature Clippy with warnings denied.
Logs: `/tmp/spaceport-qnz9/2njz-host-clippy-source-stable.log` and `2njz-windows-clippy-source-stable.log`.
Native glyph, visual, keyboard, and accessibility qualification remain pending.
