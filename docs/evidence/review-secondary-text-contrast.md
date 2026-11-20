# Secondary text contrast

Verified on macOS with Rust 1.99.0 on 2026-11-17.
Scope: the remaining secondary-text portion of `8zvq` under `9s44`.
`8zvq` is implemented and independently reviewed.
Native palette qualification completed on published source `8a4beb1ec` on 2026-11-19.
The OS preference work is recorded in [OS accessibility evidence](review-os-accessibility.md).

## Cause and change

Derived muted text used a fixed 55% background/foreground blend.
Subtext used the colored-mark threshold, which allowed 3:1 in standard mode.
Those choices left small labels below the requested 4.5:1 text floor.

One shared chrome-background helper now serves separate mark and text thresholds.
Secondary text always uses 4.5:1 against the background, surface, and hovered-row color.
The existing hue-preserving correction handles default, derived, and styled authored muted colors.
Marks retain their existing standard/high thresholds of 3:1 and 4.5:1.
High mode retains its selected-row treatment and remains idempotent.
No terminal palette, primary color, font, or row geometry changes were added.

The startup-error regression also found that fallback colors bypassed OS high contrast.
Fallback now styles the default palette through the normal configuration boundary.
It still reports the original configuration error.

Two existing caller assertions compared displayed palettes with raw authored colors.
They now compare against the styled palette while retaining their ownership and font-size assertions.

## Actual prepared-color regression

The test loads each shipped palette through `Config::theme` in standard mode.
It calculates WCAG ratios from the resulting sRGB tokens.
It checks muted and subtext against background, surface, and hover colors.

| Shipped palette | Muted on hover before | Subtext on hover before | Both text roles after |
| --- | ---: | ---: | --- |
| GitHub Light Default | 2.884:1 | Already passed | At least 4.5:1 on all three backgrounds |
| Atom One Light | 2.684:1 | Already passed | At least 4.5:1 on all three backgrounds |
| Gruvbox Light | 2.433:1 | 4.450:1 | At least 4.5:1 on all three backgrounds |
| One Half Light | 2.463:1 | Already passed | At least 4.5:1 on all three backgrounds |

The regression failed before the correction and passes afterward.
Its red log is `/tmp/spaceport-qnz9/8zvq-ratios-red.log`.
The startup fallback test independently failed with Standard instead of the requested High level.
Its red log is `8zvq-fallback-red.log` in that directory.
It now checks both OS levels and the fallback text ratios.

These are prepared-color measurements, not measurements of native screenshots or antialiased glyph edges.

## Verification

Cargo commands held the task's shared lock and used its shared target directory.
They set `CARGO_INCREMENTAL=0` and `RUSTUP_TOOLCHAIN=1.99.0`.
Compilation used:

```sh
cargo test -p sheprd-gpui --bin spaceport --no-run --message-format=json --locked
```

The throwaway runner reused `crates/test-support/sandbox.rs` for every test process.
It isolated HOME, configuration, and socket paths.
The compiled test binary received these filters:

| Filter | Passed | Other |
| --- | ---: | --- |
| `config::` | 312 | One existing ignored test |
| `app::` | 21 | Includes fallback contrast |
| `settings_window::` | 251 | Fresh rerun after the separate Settings font-scale fix |
| `settings_panel::` | 14 | Styled authored-color expectations retained |
| `theme_picker::` | 17 | Includes async OS preview and cancellation |
| `log_window::` | 16 | Includes Logs-only OS contrast |
| `contrast::` | 16 | Includes independent WCAG reference ratios |

```sh
cargo clippy -p sheprd-gpui --bin spaceport --tests --locked -- -D warnings
```

Clippy passed. Scoped formatting, whitespace, and source-size checks passed.
`just check-file-size check-sorted` also passed, including the sort checker's ten tests.
Root reviewed the text floor, palette sources, startup fallback, and regression evidence without remaining findings.
The ignored native-font alias test requires an explicitly selected installed family alias.
The actual scripts are `/tmp/spaceport-qnz9/check-secondary-text.sh` and `check-secondary-text-settings.sh`.
The final Settings rerun and Clippy logs are `8zvq-settings-final.log` and `8zvq-clippy.log` there.
Other passing group logs remain under `secondary-text-consumers/`.
The initial broad batch recorded the separate font-scale regression before its owner corrected it.
Successful unaffected groups were not rerun during that correction.

Root's `yftv` change made `focus_follows_mouse = false` the default.
Its existing preference round-trip fixture still tested false as a nondefault value.
Root authorized changing that exact fixture to true, including its pinned text and true loader assertion.
That producer/load contract passes in the 312 configuration tests.
This correction changes no focus behavior.

Native screenshot qualification remains pending.
Keyboard and working-session performance checks belong to separate findings.
No private configuration, OS preference, runtime, or daemon was changed by these checks.

## Native palette qualifier, November 19

The Windows integration fixture loads the four shipped palettes through `Config::theme_named`.
It retains the actual standard OS contrast policy and refuses a high-contrast host.
It paints muted and subtext on background, surface, and hover colors.
Each cell includes real 9px and 12px glyphs and an opaque foreground probe.

The fixture reads back only its own synthetic GPU windows.
It removes each window before encoding or measuring the image on the background executor.
Palette loading also runs on the background executor.
Each image stays within two million pixels and 128 KiB of encoded PNG data.
No daemon, desktop capture, clipboard, personal configuration, or preference write is involved.

Measurements require native RGB agreement with the configured colors.
They require an unrounded ratio of at least 4.5:1 for every text/background pair.
Separate ink counts require visible glyphs in both small-text rows.
The PNG is emitted before a failed measurement propagates, preserving diagnostic evidence.

The [WCAG contrast guidance](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html) uses foreground and background colors rather than antialiased glyph edges.
It also prohibits rounding a below-threshold ratio up to the threshold.
The solid probes measure those native colors; glyph checks and image inspection assess actual small text separately.

Independent review finds no material ownership, privacy, measurement, or original-scope gap in the fixture.
Both host and Windows GNU lint configurations and the static checks pass.
The prepared-palette and contrast consumer scope passes ten tests, with none ignored.
Command: `cargo test --locked -p sheprd-gpui --all-features --bin spaceport -- config::tests::secondary_text:: contrast::tests:: contrast::system_tests:: --nocapture`.
Log: `/tmp/spaceport-9s44-native-contrast-consumers.log`.
The existing debug-linker unwind-size warning and `block 0.1.6` future-compatibility notice remain unchanged.
Actual Windows execution, all four PNG inspections, and the measured ratios remain pending.
Kata `8zvq` stays open until that native evidence passes.

## Qualified native result on `8a4beb1ec`

The [Windows ARM native job](https://github.com/2389-research/spaceport/actions/runs/44534448320/job/135596374844) succeeds on `8a4beb1ec29a57c00368948cda2c602e24566e6c`.
The palette fixture passes at `2026-11-19T13:38:52.7347722Z`.
Its native GUI suite passes seven cases, with none ignored.
This result does not claim ARM UIA qualification; that step remains skipped by existing policy.
The complete CI run and x64 native job were still running when this result was recorded.

All 24 unique native measurements match the configured RGB values exactly.
Repeated tail summaries match their original measurements and are deduplicated.
Every background, surface, and hover pair exceeds the unrounded 4.5:1 threshold.
Every 9px and 12px glyph region has more than 300 matching ink pixels.

| Palette | Muted on hover | Subtext on hover | PNG bytes | PNG SHA-256 |
| --- | ---: | ---: | ---: | --- |
| GitHub Light Default | 4.5467134:1 | 6.085221:1 | 45936 | `8b26b81d93a517ea74ea0ab07ad7aadc3265358738d42c344288182db8312745` |
| Atom One Light | 4.54313:1 | 5.313711:1 | 45090 | `6dc7ec240599e85ae50069250d85834a1c55ef68951831b2c5496d4be5e65bdb` |
| Gruvbox Light | 4.5410166:1 | 4.5030737:1 | 44724 | `54ae83ea403db495528823a6f93bb36c0a711f747b12b499a4773082bbc3355e` |
| One Half Light | 4.505819:1 | 4.5690207:1 | 44907 | `fdd1e9c4efb63f92b374023eec59918f92b4abe5fdf7fef03cfd144ba052b5f0` |

All four full PNGs passed canonical base64, size, signature, and complete IEND checks.
The owner inspected every image: both text sizes are visible and neither column is clipped.
Each image shows the actual three background fills and both secondary text roles.
Images and the complete measurement manifest remain in `/tmp/spaceport-9s44-native-contrast-8a/arm/`.

The separate [OS-reader qualification](review-os-accessibility.md#reorder-extraction-follow-up) records actual macOS and Windows getter execution.
The original issue requires no foreground keyboard or working-session performance check.
Root checked its fresh revision-one scope against these results and the earlier independent implementation review.
No original contrast requirement remains unmet.
Kata `8zvq` closes at `2026-11-19T13:44:29.862Z`.
Closure event: `9791` (`01M7T0TED6JFV7VBP09Q40ZX7E`).
Fresh issue and parent reads retain the open umbrella, with 46 closed children and seven open children.

The [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44534448320/job/135596374736) also passes all eight GUI cases.
Its 24 native RGB pairs and ratios match ARM exactly, including the 4.5030737:1 minimum.
Raster glyph counts differ across architectures; the x64 images are not claimed byte-identical or separately inspected.
That job later fails an unrelated chat traversal deadline, with eleven UIA cases passing and one failing.
The complete CI run remains unqualified while those later results are pending.
