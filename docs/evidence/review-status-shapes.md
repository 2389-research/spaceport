# Default status shapes

Date: 2026-11-17. Scope: `05be` under `9s44`.

The default Dots style now draws Blocked as a filled square.
Done remains a filled circle. Idle remains hollow. Unknown remains smaller.
Working keeps its existing moving ring.
The shared marker renderer supplies sidebar and tab markers.
Their sizes, colors, and reserved layout slots remain unchanged.

The regression failed when Done and Blocked used the same radius.
It now passes at three marker sizes.
A headless paint regression also checks actual production marker quads at two font sizes.
Its baseline failed because Blocked painted rounded corners.
The fixture accepts GPUI's separate square border quads instead of assuming one primitive per marker.
All six agent presentation tests passed.
Root reviewed the shared renderer and its callers.
Native visual qualification remains pending.

Logs: `/tmp/spaceport-9s44-status-shapes-{red,green}.log` and `/tmp/spaceport-9s44-status-paint-{red,green}.log`.

## Native Windows qualification plan

Render the shared production marker in owned native windows at default and doubled sidebar font sizes.
Use identical marker colors against dark and light backgrounds, so color cannot distinguish the statuses.
Require visible pixels for both marks and a larger filled area for Blocked.
Return only those small authored scenes as PNG receipts through the existing isolated fixture log.
Inspect the native results and images before closing `05be`.
This check does not qualify foreground performance or the complete sidebar layout.

The existing default-style and painted-quad regressions remain the baseline proof.
The Windows-only raster fixture cannot run on this Mac; native pixels and images remain pending CI.
Both host and Windows GNU lint configurations pass.
Format, file size, sorted regions, operations, localization, and diff checks pass.
Independent review confirms that readback receives only the owned rendered scene, not desktop pixels.
The parent requires a native completion receipt and already retains the private child's complete log.
Logs use `/tmp/spaceport-9s44-native-shapes-{lint,static}.log` and the corresponding `final-windows-lint` and `final-static` logs.

## Native fixture redraw correction

Source `fa0cbb74a` fails both Windows native jobs during marker drawing.
The typed window update holds the marker view borrowed while drawing borrows that view again.
The native error names `native_shapes::Mark` as already being updated.
Drawing now uses `AnyWindowHandle`, matching the existing native confirmation fixture.
The typed handle remains available for owned-window removal before readback errors propagate.

A headless regression invokes the actual qualifier and reproduces the same borrow panic before the fix.
It then requires the expected unavailable-renderer error and the original open-window count for both backgrounds.
This checks redraw and cleanup; it supplies no native pixel evidence.
The red log is `/tmp/spaceport-9s44-native-shapes-borrow-red.log`.
The corrected agent-marker scope passes nine tests with no failures or ignored tests.
Both host and Windows GNU lint configurations pass after the correction.
Format, file size, sorting, operations, localization, and diff checks pass.
Independent review confirms the borrow correction and owned-window cleanup.
The final logs use `/tmp/spaceport-9s44-native-shapes-borrow-{green,lint,static}.log`.
Native status-shape qualification remains pending.
