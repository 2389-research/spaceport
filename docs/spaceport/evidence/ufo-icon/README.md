# Nebula saucer verification

Doctor Biz selected 🛸 on November 13, 2026. D68 records the artwork change.
The selected master, `design/spaceport-icon/ufo-nebula.png`, retains the nebula tile.
The repository's `design/spaceport-icon/README.md` preserves the exact built-in imagegen prompt and source hash.
The old rocket master remains a historical reference with its existing legal provenance.

All platform SVGs embed the selected PNG's exact bytes.
The existing masks, development rim, bundle identities and catalog key remain.
The generated macOS catalogs use the internal `spacecraft` image layer.

## Real native captures

These images show the actual About popup in isolated native app processes.
They use synthetic fixture data, private HOME/XDG paths and no personal daemon.
The before build uses the four PNG/ICNS assets from `79cc51d30`.
The after build uses the saucer assets with the same fixture code.
Both captures wait for the popup's actual enter animation before saving the frame.
The fixture also checks text clipping, Escape, focus restoration and real startup diagnostic capture.

| Theme and width | Before | After |
| --- | --- | --- |
| Dark, 800 | [Before](before/about-dark-800.png) | [After](after/about-dark-800.png) |
| Dark, 360 | [Before](before/about-dark-360.png) | [After](after/about-dark-360.png) |
| Light, 800 | [Before](before/about-light-800.png) | [After](after/about-light-800.png) |
| Light, 360 | [Before](before/about-light-360.png) | [After](after/about-light-360.png) |

Run from the repository root with a writable absolute capture directory:

```sh
mkdir -p "$HOME/spaceport-captures"
HERDR_TEST_LOCAL_CONTROLS_CAPTURE="$HOME/spaceport-captures" \
  cargo test --locked -p sheprd-gpui --features integration-test \
  --test live_gui native_local_controls -- --ignored --exact
```

## Artifact checks

The old exports failed the selected-master and Linux metadata regressions before regeneration.
`just check-artwork` passes all ten tests.
`uv run --no-project python -m unittest discover -s scripts/release/tests -p 'test_linux_icon*.py'` passes nine tests.
These cover exact master bytes, reproducible generation, masks, geometry, negative corruption cases and saucer clearance.
Existing coverage remains after renaming the rocket test module to `test_spacecraft.py`.

Regeneration ran these existing producers in order:

```sh
uv run --no-project python scripts/generate-linux-icons.py
uv run --no-project python scripts/generate-windows-icons.py
uv run --no-project python scripts/generate-installer-images.py
swift scripts/generate-icons.swift
uv run --no-project python assets/branding/generate.py
```

`swift scripts/check-icons.swift` passes native decoding and iconutil round trips.
Both ICNS files contain all ten standard/Retina representations.
Both catalogs contain the selected image and light/dark appearances.
The native checks preserve the small-size disc/canopy visibility and margin assertions.
Windows container checks preserve all eight ICO sizes; no native Windows run is claimed.

`just format-check check-file-size check-sorted check-i18n` passes.
The schema-backed specification check passes after regenerating the decision views.
`cargo test --locked -p sheprd-gpui --features integration-test --bin spaceport app_icon::tests` passes both AppKit/icon tests.
`cargo clippy --locked -p sheprd-gpui --all-targets --features integration-test -- -D warnings` passes.
Rust checks use 1.99.0, `CARGO_INCREMENTAL=0`, `HERDR_BUILD_PR_NUMBER=''` and the existing display-copy target.
Cargo retains the existing `block 0.1.6` future-incompatibility warning.
Independent review found no material regression in the artwork or capture changes.
Detailed command logs remain in the ignored `target/ufo-icon/` directory.
No signed build, installation, published release or native Linux/Windows qualification is established by these checks.
