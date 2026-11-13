# Branding audit follow-up evidence

These captures show real native Settings windows from private HOME/XDG fixtures.
They contain synthetic default settings, with no daemon or personal installation.
The fixture renders without requiring Doctor Biz to unlock the desktop.
It does not qualify OS key-window activation or Linux window grouping.

The `before` captures use the old Fluent label with the new capture helper.
The `after` captures use `Spaceport / workspace`.
Both sets cover dark and light themes at 960 and 680 logical pixels.
The PNGs contain the full native window at the display's pixel scale.

| Theme and width | Before | After |
| --- | --- | --- |
| Dark, 960 | [Before](before/fonts-dark-960.png) | [After](after/fonts-dark-960.png) |
| Dark, 680 | [Before](before/fonts-dark-680.png) | [After](after/fonts-dark-680.png) |
| Light, 960 | [Before](before/fonts-light-960.png) | [After](after/fonts-light-960.png) |
| Light, 680 | [Before](before/fonts-light-680.png) | [After](after/fonts-light-680.png) |

Run the native fixture from the repository root:

```sh
mkdir -p "$HOME/spaceport-captures"
HERDR_TEST_LOCAL_CONTROLS_CAPTURE="$HOME/spaceport-captures" \
  cargo test --locked -p sheprd-gpui --features integration-test \
  --test live_gui native_local_controls -- --ignored --exact
```

The fixture also verifies local actions, popup isolation and focus restoration.
It flushes diagnostics through the background executor before the app exits.
The harness requires the actual GUI startup event in the fixture's private log.
The old GUI filter failed that assertion; the compiler-derived filter passes it.

See [the execution record](../../notes/branding-audit-followups.md) for checks and platform limits.
