# Native updater qualification, November 11

This records real signed fixtures, not a published Spaceport update. The frozen
GPUI fixture executable SHA-256 is
`c9c19fc29002bffe9f4c6d6983af28e3f71ce923968a3b427fbb91113347b818`.
Sparkle is the pinned 2.10.1 SDK. Each case has fresh random bundle IDs, disposable
Ed25519 keys, private state and exact-process cleanup. Evidence roots below are
retained under the Sparkle worktree's ignored `target/sparkle-native/`.

## Passed with cache ownership checks

| Case | Evidence root | Archive requests |
| --- | --- | ---: |
| Wrong archive key | `wrong-archive-key-zmfqcxyw` | 1 |
| Altered ZIP | `altered-zip-5qbqzdoc` | 1 |
| Wrong payload name and ID | `wrong-name-id-jts7obe6` | 1 |
| Inner payload version rollback | `payload-downgrade-8k3aur5h` | 1 |
| Altered signed feed | `altered-feed-e11_ymba` | 0 |
| Wrong feed key | `wrong-feed-key-all85077` | 0 |
| Wrong feed key after 21-day failure state | `aged-wrong-feed-key-ewbr3kzh` | 0 |
| Older advertised version | `downgrade-8b8fb65b` | 0 |

The four archive cases used genuine native Install Update buttons and showed
native refusal dialogs, acknowledged with Cancel Update. The driver requires
the exact native domain/code/cause and ordered download evidence. Each case
preserved executable/plist hashes and the original live process before cleanup.
Feed/selection cases use the native information-check API and downloaded no
archive. All eight exited through their private quit sentinel and passed exact
preclaimed account/private cache cleanup and scoped preference cleanup.

The payload rollback offered version 3 to a version-2 host, while the signed
inner app contained version 1. Its native underlying error was 4006 and named
the refused downgrade from 2 to 1. This exercises inner payload validation
separately from appcast version selection.

## Positive results and remaining checks

Earlier `valid-zip-hc7bv5ja` and `valid-dmg-aqdmz042` both installed/relaunched v2
at the expected executable path and bundle ID, passed strict code-signature
verification and quit normally. They predate cache ownership claims and do not
qualify the new positive cleanup path.

`valid-zip-bqhfk3qk` replaced v1 with v2 but recorded two new v2 processes. Scoped
OS logs show two separate LaunchServices open requests within milliseconds.
The final UI call also observed the app during replacement; that observation
may have opened the bundle again, but the initiating callers are unproven.
The case remains failed. The next fresh run must retain exactly one relaunch;
observe global app inventory after the final click rather than retargeting the
bundle during replacement. No thresholds or lifecycle checks were relaxed.

`valid-zip-0lanzk01` and `valid-dmg-7k5dcw9v` found v2 but timed out after the
desktop locked before native installation. Both retained failed results and
cleaned up normally. A separate DMG preparation timed out creating its disk
image; a fresh preparation subsequently succeeded. Failed roots remain intact.

Fresh cache-safe ZIP/DMG positives and the same-name/different-ID publisher
rotation case remain pending. Public preview publication, signed-feed bootstrap
and the real update from Doctor Biz's installed `20261111.1` baseline also remain
pending. T12 and A43 stay open.

## Test portability

The final driver suite initially ran 54 tests: 53 passed and one real ownership
change skipped because it requires privileges. CI then exposed that framework
Python can exec outside its copied launcher path. The test now compiles a tiny
native helper and asserts its actual executable path before checking refusal.
The cache topic passes under both local Python runtime shapes: 12 passed and
one unchanged ownership skip each. The production census/cleanup rules did not
change. Full exact-source platform CI remains required.
