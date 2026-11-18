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
