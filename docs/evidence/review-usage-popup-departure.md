# Usage popup departure

The closing Usage panel captures its observation before the active menu resets.
It retains provider tabs, host text, status text, report time, and numeric scroll placement.
Later observations cannot change its report or tabs.

Live drawing borrows the current report.
Only dismissal copies the report's standard fields and supported concrete details.
The snapshot excludes the report's opaque `Any` value.
It copies `OpenaiDaily` and `ClaudeAccount` data through their existing concrete types.
The current and captured reports share the same detail templates.

The active and outgoing panels share their header, body, links, and tab templates.
Outgoing links and tabs register no handlers or hover effects.
Outgoing content retains no scroll or focus handle.
The captured time also keeps countdown and pace text stable while closing.

## Verification

The actual regression first painted Claude and Codex tabs.
After closing, it replaced the observation with Grok and Cursor.
The original code lost the Codex tab while departure still painted.
Red: `/tmp/spaceport-9s44-usage-departure-red.log`.
The immutable red executable is `/tmp/spaceport-9s44-usage-red-tests`.

Initial green checks use `/tmp/spaceport-9s44-gpui.WXDVAO/spaceport-tests`:

```sh
/tmp/spaceport-9s44-gpui.WXDVAO/spaceport-tests usage::tests --test-threads=1
/tmp/spaceport-9s44-gpui.WXDVAO/spaceport-tests usage::paint::tests --test-threads=1
/tmp/spaceport-9s44-gpui.WXDVAO/spaceport-tests menu::appear::tests::usage --test-threads=1
```

All eighteen Usage consumers and both report-owner tests pass.
The original actual-window regression passes.
The detached link and tab fixture has no parent input shield.
It preserves the replacement panel and records no opened URL.
The opaque-owner check verifies resource release after the source observation drops.

Logs: `/tmp/spaceport-9s44-usage-consumers-green.log`,
`/tmp/spaceport-9s44-usage-owners-green.log`, and `/tmp/spaceport-9s44-usage-first-green.log`.

The additional actual scroll-position case passes in the next immutable artifact:

```sh
/tmp/spaceport-9s44-gpui.Dl8yq5/spaceport-tests menu::appear::tests::usage::departing_usage_content_keeps_its_real_scroll_position --exact --test-threads=1
```

Its exact panel and header positions match the scrolled active panel.
Log: `/tmp/spaceport-9s44-usage-scroll-green.log`.
The two artifacts contain the same Usage production source.
Together, these checks qualify 22 cases.

Independent lifecycle review approved the snapshot data, shared templates, and live borrowing.
The final read found no material issue.
Formatting, source-size, sorted-region, i18n, and diff checks pass at this boundary.
I18n validates 2,174 messages in 114 files.
Fresh aggregate host and Windows lint await the remaining popup families.
Native visual, input, and accessibility qualification remain pending.
