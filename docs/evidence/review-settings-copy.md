# Settings descriptions

Date: 2026-11-17. Scope: `p9wk` under `9s44`.

The copy now explains notification delivery, copy-on-select, test sounds, and file precedence.
The notification description names the Show toasts override because it can override Terminal and Off delivery.
The copy-on-select description explains when Ctrl-C copies instead of interrupting.
The sound description distinguishes notification sounds from explicit tests.
Position and delay descriptions name the actual private files instead of saying managed-file override.

Root checked each claim against input handling, sound playback, notification resolution, and config writers.
The internationalization, sorted-region, and source-size validators passed.
No runtime behavior changed, so no new behavior tests were needed.

The isolated native local-controls harness passed in both themes after loading its private view settings.
It captured Notifications and Copy on select at 680 and 960 pixels.
Root inspected the narrow descriptions and volume controls in the resulting native captures.
The changed descriptions remain visible and wrap inside their rows.
See [settings captures](review-squad/settings-copy/).
The broader native sidebar harness could not activate its decoy window on this desktop.
That failed check is not reported as passed.

The General page has an existing narrow row above the captured clipboard controls that wraps a word vertically.
This copy change did not alter that row.
The separate [settings ownership fix](review-settings-ownership.md) now corrects those labels.

Logs: `/tmp/spaceport-9s44-root-quick.log`, `settings-copy-native.log`, and `settings-copy-loaded.log`.
