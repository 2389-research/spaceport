<!-- ABOUTME: Records Devices picker and session-form departure ownership and runtime checks. -->
<!-- ABOUTME: Separates prepared display verification from pending native and platform qualification. -->
# Devices picker and session forms

## Scope

Capture Devices display before the active popup resets.
Keep discovery, selection, and session mutation controllers unchanged.
Keep accepted scans and mutation tasks in app-owned state.
Retain no endpoint, session action target, input entity, task, or scroll handle in paint.

`DevicesPaint` contains picker display and optional Create/Delete form display.
Its `session_modal()` flag preserves the existing dialog geometry.
The modal keeps the captured picker underlay beneath the fading backdrop.
Shared active and passive templates preserve one source of layout.
Passive controls omit callbacks, hover styling, pointer cursors, and actionable accessibility roles.

Picker blocks retain measured direct-child bounds relative to the existing viewport and numeric scroll offset.
Live indexed children remain direct children of their ScrollHandle.
Partial session disclosure uses measured clipping, rather than inferred row heights.
Activity display stores aggregate counts without endpoint history identities.
Core display retains bounded machine and group text without selection identities.
Create stores `SearchInput::Paint` and releases the original input entity through normal reset.

## Original regression

The original runtime test measured the picker and both session form bodies.
After dismissal, it cleared cached session observations and released the old input owner.
All three original bodies disappeared.

- Binary: `/tmp/spaceport-9s44-gpui.WhxrQX/spaceport-tests`.
- Log: `/tmp/spaceport-9s44-devices-picker-red.log`.
- Result: one failed test containing all three body failures.

## First integrated result

Binary: `/tmp/spaceport-9s44-gpui.ZcCcNN/spaceport-tests`.
Runner: `/tmp/spaceport-qnz9/run-accessibility-consumers`.
Summary: `/tmp/spaceport-qnz9/ydr4-devices-integrated.log`.

| Filter | Passed | Failed |
| --- | ---: | ---: |
| `menu::appear::tests::devices_picker` | 1 | 0 |
| `menu::devices::tests` | 23 | 0 |
| `menu::sessions::management::tests` | 6 | 0 |
| `devices_overview::tests` | 32 | 0 |

The original body checks now pass after reset.
The old Create field releases.
These counts predate the expanded scroll and detached-control tests.

## Expanded regression

Binary: `/tmp/spaceport-9s44-gpui.xD8o8D/spaceport-tests`.
Summary: `/tmp/spaceport-qnz9/ydr4-devices-expanded.log`.
The actual list scroll test passed after an 80-pixel wheel event and observation replacement.
Both detached-control tests and accepted-task ownership passed.

The long-error modal test reproduced an exact 80-pixel departure jump.
The active form moved from y23 to y-57 after scrolling.
The frozen form returned to y23.
The fix tracks a dedicated live session-form ScrollHandle.
`EditPaint` copies its numeric y offset and applies it once beneath the frozen shell clip.
The paint retains no handle or Edit owner.

Two expanded fixtures failed before their departure assertions.
The partial-height fixture assumed unsnapped full-height division.
The corrected precondition checks a nonzero partial clip and retains exact measured departure equality.
The Core fixture supplied a label above the document's 200-character limit.
It now uses that limit and asserts a Ready document before capture.
The corrected fixtures and modal offset passed against `/tmp/spaceport-9s44-gpui.VP8SIj/spaceport-tests`.
Summary: `/tmp/spaceport-qnz9/ydr4-devices-offset.log`.
Appearance passed three tests; private departure passed five tests.
These include exact partial-open/fold geometry, Core observation changes, detached controls, and accepted-task ownership.

Root review requested a new per-form lifetime regression.
The test scrolls a Create form by 80 pixels, sends Escape, and opens another prepared form without submitting.
The next Create inherited y-57 instead of starting at y23.
The fixture retains an attach-only local socket and uses inert remote display facts.
The fix resets the dedicated display handle in `cancel_session_edit` alongside releasing the Edit owner.
The same regression also checks a subsequent Delete form.
Against `/tmp/spaceport-9s44-gpui.HPbII0/spaceport-tests`, the next Create now starts at its original top.
The Delete loop then stopped at its armed-confirmation precondition.
The fixture had reused the saved profile's session name, which deletion correctly protects.
The fixture now uses a different saved profile session and retains the cached deletion candidate.
The final qualification below covers both Create and Delete transitions.

The same clean-member artifact passed appearance 3, full Devices 28, and overview 32 tests.
Six existing management consumers also passed.
Summary: `/tmp/spaceport-qnz9/ydr4-devices-final.log`.

New fixtures exercise actual list scrolling, long labels, and long-error modal scrolling.
Partial opening and folding use held progress with the production disclosure frame.
Core catalog fixtures replace the selected machine and document observation after capture.
They check old group bounds and preserve a different current machine selection after a frozen row click.
Detached controls render without a parent input shield.
Remote session fixtures remove the exact endpoint identity before clicking stale rows.
They never click an uncontained local session row.
Replacement forms use invalid names and an explicit nonexistent attach-only socket.
The fixtures never call discovery, accepted session mutations, or daemon startup.
An accepted-task Drop probe checks that capture and reset preserve app ownership.
That probe performs no session mutation or remote write.

Quick owned Rust formatting, file-size, sorted-region, and diff checks passed after the cancellation reset.
`just check-i18n` passed with 2,174 messages in 114 files.

## Final qualification

Binary: `/tmp/spaceport-9s44-gpui.HsLaYo/spaceport-tests`.
The root built this immutable pin after cleaning changed workspace members and qualifying aggregate lint.

```sh
/tmp/spaceport-qnz9/run-accessibility-consumers \
  /tmp/spaceport-9s44-gpui.HsLaYo/spaceport-tests \
  /tmp/spaceport-qnz9/ydr4-devices-qualified \
  menu::appear::tests::devices_picker \
  menu::devices::tests \
  menu::sessions::management::tests \
  devices_overview::tests
```

Exit 0. Summary: `/tmp/spaceport-qnz9/ydr4-devices-qualified.log`.

| Filter | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| Devices appearance | 3 | 0 | 0 |
| Devices consumers and departure | 28 | 0 | 0 |
| Session management and fresh-form scroll | 7 | 0 | 0 |
| Devices overview | 32 | 0 | 0 |

The cancellation test executes both subsequent Create and subsequent Delete cases.
The accepted-task test confirms capture, reset, and paint disposal retain the app-owned task.
Its explicit owner then releases the task, and the Drop probe fires.
No fixture performs an accepted session mutation or remote write.

Root ran host and Windows GNU Clippy across all targets and all features with warnings denied.
Both commands exited 0; their logs finish successfully.
Logs: `/tmp/spaceport-9s44-host-clippy-qualified.log` and `/tmp/spaceport-9s44-windows-clippy-qualified.log`.
The host log retains the existing `block` 0.1.6 future-compatibility notice.

Root independently approved the final local lifecycle and render scope after inspecting source and qualified runtime results.
The review confirmed numeric offset capture before reset, one passive application, and display-only cancellation reset.
It covered detached controls, field release, Core observation replacement, and accepted-task ownership.
Root also qualified all 65 appearance tests after the final passive DTO boxing changes.
The parent motion issue remains open while native UI qualification is pending.
Native desktop, accessibility technology, OS keyboard/IME, and release performance checks remain pending by Doctor Biz's instruction.
Headless geometry checks do not qualify native rendering or input.
