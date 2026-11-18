# Secondary text contrast

Verified on macOS with Rust 1.99.0 on 2026-11-17.
Scope: the remaining secondary-text portion of `8zvq` under `9s44`.
`8zvq` is implemented and independently reviewed.
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

Native screenshots, keyboard checks, and performance checks remain pending at Doctor Biz's request.
No private configuration, OS preference, runtime, or daemon was changed by these checks.
