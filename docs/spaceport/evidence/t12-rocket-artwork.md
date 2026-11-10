# T12 — Spaceport rocket artwork

This records the earlier flat emoji phase. The approved production artwork and
later export checks are in [the nebula artwork evidence](t12-nebula-artwork.md).

Doctor Biz selected the rocket emoji in D62. This change starts from
`6ba05b0ab` on `wip/spaceport-rocket-branding` and is limited to artwork,
its generators/checks, branding exports and supporting documentation.

`assets/icons/sheprd-avatar.svg` holds the pinned Twemoji U+1F680 geometry,
scaled onto a navy gradient. NOTICE records the source, authors, CC BY 4.0
license link and placement changes; LICENSE-twemoji holds the full license.
Development variants retain the existing berry background and the same emoji
colors. Existing asset filenames and catalog keys keep current packaging and
embedded references connected to the regenerated artwork.

## Verification

- Baseline `just check-artwork`: four tests passed.
- Added rocket regressions and ran `just check-artwork` before implementation:
  failed on the old Sheprd title and paw-specific generator validation.
- `just icons`: generated Linux SVGs, Windows ICOs/installer images, macOS
  ICNS/catalogs and embedded PNGs successfully.
- `uv run --no-project python assets/branding/generate.py`: generated the five
  Spaceport social cards; all remained below the existing upload-size limit.
- `just check-artwork`: seven tests passed, covering shared-source derivation,
  emoji geometry/colors, development background, malformed source, Windows
  container sizes and social-card dimensions.
- `swift scripts/check-icons.swift`: both ICNS variants passed all ten native
  representations, including 16px hull/flame visibility; embedded PNG bounds
  and both compiled catalogs' rocket vectors/light-dark entries passed.
- `just check-file-size`, `just check-sorted`, `git diff --check`: passed.
- `just format-check`, changed Python files' AST syntax checks, and
  `actionlint .github/workflows/ci.yml`: passed.
- Specification `tools/spec.py check`: passed with schema validation.
- Inspected the exported app icon and light social card visually. Reviewed the
  source changes, artwork provenance, generator paths and retained catalog keys.
- The independent reviewer caught paw-specific assumptions in the release icon
  tests, lost canonical-filename metadata and a missing-path validation gap.
  Reproduced the failures, restored validation/provenance and extended the
  independent geometric crop check for nested transforms and smooth cubic paths.
  Its corrupt-layout cases remain, with an added inner-transform case. Seven
  Linux icon tests passed; the second review found no remaining issues.
- Full `python3 -m unittest discover -s scripts/release/tests` with
  `PYTHONDONTWRITEBYTECODE=1`: 280 tests, no failures, eight skips. These are
  the existing release fixtures, including mocked packaging tools; they do not
  prove native installation or a signed Spaceport build.

## Limits

No Spaceport application build, signing, installation or native window check
was performed. Existing signed previews still contain their original artwork.
Windows/Linux desktop rendering and full workspace tests/lints remain for CI
and native QA. This is asset/export evidence, not A42–A44 or A49–A51 acceptance;
T12 and its release/platform gates remain open. No commit, merge, push, upload
or publication was performed.
