# Provider polling and sign-in implementation removal

Final code snapshot: `2d2f11a0b794144687f0d7f9b9801a914f191a99`, following
initial cut `cb9b6a97c44a10b69a0c340f0a3a95600594abf3` from `d96d7d9a6`.
This completes the compiled provider graph cut left by the
[earlier entry-point removal](t04-provider-probe-removal.md) and
[dependency investigation](t04-usage-tail-dependency.md). T04 remains blocked
on G-LINK and T08 on G-OBS. This evidence does not close A20 or A30.

## Removed and retained

All 87 provider fetch implementations, cookie/credential/HTTP probes,
poll/refresh/allow workers and retry/backoff state are deleted. Provider
sign-in tables, browser-cookie settings, setup examples, parser-only
translations and their tests are removed. Direct, inline, empty-table and
merged configuration forms now reject those excluded settings.

The existing crate retains report/account/window/balance/section models,
numeric and text rendering, Claude account detail and OpenAI daily chart
data, and static provider names/icons/external links in their original
order. Those records are presentation metadata; they do not advertise a
supported provider or backend feed. Display preferences remain. Shared
visibility filtering now serves the status bar, panel/section and CLI;
it does not alter stored reports. Tests distinguish a real zero from an
absent reading and preserve chosen-provider order.

The bounded host shell moved out of the probe graph. Its existing SSH
Quick lane, target validation, WSL refusal on unsupported platforms,
channel ownership, deadlines, bounded reader and child cleanup remain.
Secret capture, credential JSON helpers, cookie reads and curl builders
are gone. A deterministic test reproduced an inherited completed-response
limit bypass: a payload and completion marker arriving together could
return more than the limit. The helper now rejects that payload and
refuses further steps on the broken session. Follow-up tests also cover
every split of the completion marker after an exact-limit payload and
bounded status framing. Marker bytes do not consume the payload budget;
excess payload or status framing breaks the session.

There is no new report feed or backend RPC. UI and CLI say that no
authoritative usage feed is connected, without inferring a Herdr 1
capability from a Core-specific gap. Provider/rate-limit and automation
parity remain blocked. The generic credential-store crate has a separate
remaining dependency through `RELEASE_BUILD` and diagnostic types; this cut
does not claim that every credential-related dependency is gone.

## Verification

Tests first failed on the old compiled graph, accepted sign-in settings,
lost visibility filtering, completed oversized output and Core-specific
unavailable wording. Changes then passed:

- `cargo +1.99.0 test -p sheprd-usage --locked --offline`, with default
  and all features: 22 passed in each mode after review corrections.
- `cargo +1.99.0 test -p sheprd-gpui --bin spaceport --all-features --locked --offline`:
  3,923 passed, zero failed, nine existing opt-in tests ignored on the final
  snapshot. The default suite passed 3,893 before the final wording edit;
  final focused usage tests passed 31 and cover that edit.
- App checks with `--all-targets --all-features` and
  `--all-targets --no-default-features`, strict usage-crate Clippy, format,
  file-size, sorted-region and i18n checks passed. The i18n checker remains
  strict: parser-only messages lost their producers, while generic and
  detail-rendering localization tests remain.
- The 69-test release absence subset passed. Complete release-tool discovery
  ran 252 tests successfully with eight skips. Final development bundle
  assembly and packaged CLI checks passed as described below.
- Strict app Clippy at the initial cut remains failed with exactly 253
  inherited diagnostics and no new diagnostic groups. Strict usage-crate
  Clippy passed again after the follow-up. The successful comparison was
  not repeated for the final narrow fix. No allowance or hook bypass was added.

Cargo checks/tests use Rust 1.99.0 and `CARGO_INCREMENTAL=0`. Detailed local
logs and the implementation report use the `/tmp/provider-*` prefix; the
structured comparison is retained in
[provider-implementation-comparison.json](provider-implementation-comparison.json).
The lockfile loses 19 package/version pairs and adds/upgrades none. An
all-platform offline metadata attempt stopped at an uncached Linux-only
dependency; native checks and direct metadata passed, not a Linux build.

The inherited macOS linker unwind-size warning and `block 0.1.6`
future-compatibility notice remain. No native desktop, real runtime/host,
provider account, personal credential, Windows/Linux run or release build
was used to qualify this change. No app was launched or installed and no
PACT object was changed. The separate
[user-reported remote terminal smoke](user-remote-terminal-smoke-20261109.md)
does not identify or qualify this later snapshot.

Independent review found lost Gemini and Antigravity status-page links.
Both used multiline builder calls that the initial extractor/comparison missed.
The claim of identical metadata was therefore false for the initial snapshot.
A throwaway test called the original Rust registry's static metadata getters
to capture all five fields in order without invoking fetch or reading an
account. It passed on the pre-cut source; its capture becomes a retained
baseline fixture for the correction. It exposed two more losses: Qwen Cloud
and TypeSafe dashboard links. The temporary test was removed. The permanent
fixture test compares all five fields of every ordered metadata record;
the old regex-based comparison is superseded.

Review also identified an inherited exact-limit/partial-marker edge in the
retained shell. Both findings have test-first corrections in the follow-up
commit, and the full all-feature app suite passed again on that snapshot.
Independent re-review approved both specification compliance and task quality
after checking the corrected fixture against all original getter records and
tracing complete/partial marker bounds. No Critical or Important findings
remain in this bounded change. This does not close the separate native,
platform, authoritative-feed or release gates; task/acceptance statuses remain
unchanged. The code commits were merged into `wip/spaceport-t00`, not main.
The integrated production/test/build source tree matches the reviewed task
snapshot exactly. Integrated formatting, file-size, sorted-region, i18n and
diff checks passed; package validation, its 21 tool tests and bounded T04/T08
context packets also passed. Those package checks establish graph integrity,
not application parity.

## Development candidate

`RUSTUP_TOOLCHAIN=1.99.0 CARGO_INCREMENTAL=0 just bundle debug` passed on
`2d2f11a0b`. The copied test artifact is
`target/test-builds/2d2f11a0b/Spaceport.app`, with adjacent `build.json`,
`README.txt` and linked-library inventory. Its archive is
`target/test-builds/Spaceport-macos-arm64-2d2f11a0b.zip` (46,190,358 bytes).
The bundle is a macOS arm64 debug build, ad hoc signed and not notarized.

The copied bytes passed `codesign --verify --deep --strict`, exact bundle
identity and companion inventory, `scripts/release/smoke-cli.py`, removed
`system --json` rejection before startup, linked-framework inventory and
`unzip -t`. Only `spaceport` and `sheprd-acp` are bundled; there is no Herdr
runtime and no WebKit/JavaScriptCore framework. Hashes are in `build.json`.
This checks packaging and windowless CLI behavior, not the native GUI or a
remote session. Earlier test artifacts remain intact. Nothing was launched,
installed or published.
