# Spaceport artwork

The canonical 🚀 rocket is [the approved nebula PNG](../design/spaceport-icon/rocket-nebula.png).
Platform icons, installer artwork, and Apple app catalogs derive from it;
see [`icons/README.md`](icons/README.md) for regeneration and sizing.

## Social cards

| Source and PNG | Size / use |
| --- | --- |
| `branding/sheprd-social.{svg,png}` | 1200×630, Open Graph and X |
| `branding/sheprd-repository.{svg,png}` | 1280×640, GitHub repository preview |
| `social-preview-light.{svg,png}` | Light repository card |
| `social-preview-dark.{svg,png}` | Dark repository card |
| `social-preview-light-ui.{svg,png}` | Compatibility filename for the light card |

All cards use the rocket and the line **Your agents. One place.**
They are artwork, not screenshots. The self-contained SVGs are generated from
the approved artwork; librsvg exports each card's PNG. The embedded rocket
artwork was created with image generation, as recorded in the design notes.

```sh
python3 assets/branding/generate.py
```

Requires `rsvg-convert` (`brew install librsvg`). Exports remain below GitHub's
1 MB limit. Upload the repository image in Settings > General > Social preview.
