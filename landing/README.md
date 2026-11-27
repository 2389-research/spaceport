# Spaceport landing page

The public page for Spaceport, at <https://2389-research.github.io/spaceport/>.
It is plain HTML, CSS and JavaScript with no build step. `site/` and
`website/` hold the inherited Sheprd sites and are not published from here.

| Path | Purpose |
| --- | --- |
| `public/` | Everything GitHub Pages serves |
| `public/index.html` | The page |
| `public/assets/site.css`, `site.js` | Styles, and the download-link lookup |
| `public/assets/fonts/` | Michroma, Barlow Condensed and Spectral (SIL OFL), from Fontsource |
| `public/assets/rocket-*.webp`, `icon-*.png` | Copies of `design/spaceport-icon/rocket-nebula.png`: the webp files are cropped to the square inside its rounded edge (`magick -crop 845x845+205+205`), the icons are resized only |
| `tests/` | Link and path checks |

## Preview and test

```sh
python3 -m http.server -d landing/public 20389
python3 -m unittest discover -s landing/tests
```

## Publish

`.github/workflows/pages.yml` runs the tests, then deploys `public/` to GitHub
Pages on each push to `main` that touches `landing/`. Pull requests run the
tests only. The repository's Pages source must be set to GitHub Actions.

## Rules for the copy

- Describe what Spaceport does. Herdr runs terminals and keeps sessions; do
  not present those as Spaceport features.
- State preview status plainly. Do not claim Linux, Windows or a production
  release.
- The window on the page is an illustration and is labeled as one. Replace it
  only with real Spaceport screenshots; `docs/screenshots` is inherited from
  Sheprd.
- Keep every path relative. The page is served under `/spaceport/`.
- The download buttons ask the GitHub API for the newest preview disk image,
  because previews are prereleases. Without JavaScript they open the releases
  list. A stable release is not offered here yet: the names, labels and pages a
  stable download link will need are the stable channel's entries in
  `scripts/release/channels.json`.

Product facts live in `PRODUCT.md`; the page's design contract lives in
`.impeccable/surfaces/`.
