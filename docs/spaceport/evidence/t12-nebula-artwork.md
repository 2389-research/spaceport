# T12 — Approved nebula rocket artwork

Doctor Biz approved the sculpted rocket and nebula background in D63 and asked
to ship it. The task branch starts at `6ba05b0ab` and integrates with the existing
`wip/spaceport-t00` development branch; it does not release the whole fork.

## Source and exports

The canonical image is `design/spaceport-icon/rocket-nebula.png`, 1254×1254 RGBA,
SHA-256 `caa551b4dc59b1318f736cadc6e09c03f3ba5c8644ca4f9ed45f71d1ff4e4b00`.
The design notes preserve the built-in image generation prompts and initial
reference. NOTICE and LICENSE-twemoji retain the Twemoji concept's provenance.

The existing Python/librsvg/Swift pipeline now embeds the exact approved PNG
inside self-contained SVGs. Its measured alpha bounds, `(114,114)` through
`(1138,1144)`, remove source padding before platform margins/masks are applied.
No runtime renderer, dependency, transport, package identity or release policy
was added. Development variants add a berry rim without recoloring the image.

Native ICNS and Icon Composer catalogs, Windows ICOs/installer images, Linux
SVGs, embedded/About PNGs, README artwork and social cards use the selected
image. Existing asset filenames/catalog keys preserve their selection routes.
The macOS catalogs use a raster layer, as supported by Apple's
[Icon Composer guidance](https://developer.apple.com/videos/play/wwdc2025/361/).

The independent reviewer found that a full-size circular crop cut the nose and
plume. Linux now insets the artwork to 84% of the circle's diameter; social cards
use the rounded tile. Regression checks keep both observed landmarks at least
24 pixels inside the circular mask. Final exports were inspected visually.

## Verification

- TDD: new exact-source and malformed-input tests failed against the earlier
  vector implementation. Native ivory-hull checks also rejected the flat icon.
  The circle landmark regression failed on the original clipped export, then
  passed after the inset. A missing-gradient regression found and fixed a
  generator validation gap.
- `just icons`: regenerated all platform assets successfully.
- `uv run --no-project python assets/branding/generate.py`: all five cards
  generated within the existing size limit.
- `just check-artwork`: ten passed; approved source hash/bytes, malformed PNGs,
  reproducible exports, platform sizes and social dimensions covered.
- Linux icon discovery: nine passed; independent mask, mapping, development
  rim, corruption, missing-content and nose/plume clearance checks covered.
- `swift scripts/check-icons.swift`: both ICNS variants passed all ten standard
  and Retina representations, including 16px ivory hull/flame visibility.
  Embedded PNG dimensions/margins and both raster catalogs' light/dark entries
  passed.
- Release fixture discovery: 282 tests, no failures, eight skips. Existing
  packaging stand-ins verify scripts rather than native installation.
- `just format-check`, `just check-file-size`, `just check-sorted`,
  `actionlint .github/workflows/ci.yml`, specification render/check with schema
  validation, and `git diff --check`: passed.
- Independent review found no remaining blockers after the missing-gradient
  and circular-crop fixes. Final source/provenance/export review passed.

## Boundaries

This is artwork/export verification. No new Spaceport binary was built, signed,
notarized, installed or launched. Previously signed previews retain their old
artwork. Full workspace/platform checks and native Dock/About verification are
deferred to CI and the next application preview. T12 and its release/platform
acceptance gates remain open; no public release or updater was enabled.
