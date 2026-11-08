# Inherited sheprd.org site

This is the inherited Sheprd marketing site, not a Spaceport release surface.
`website/netlify.toml` skips Git-triggered builds in this fork. The source is
plain HTML, CSS and JavaScript, with no install or build step. `site/` is
separate and stays with herdrui.dev. No owner-controlled Spaceport site or
publish route has been established.

```sh
python3 -m http.server -d website/site 8000
```

Python's server does not emulate Netlify headers or redirects. A Netlify
project with base directory `website` reads `website/netlify.toml`, whose
publish folder is `website/site`. This README and the config sit outside
that folder.

`herdrui/` is the separate Cloudflare Worker for herdrui.dev, and is not
published by Netlify; see [`herdrui/README.md`](herdrui/README.md).

`site/install.sh`, `site/install.ps1` and `site/llms.txt` here must match the
copies in the root `site/`,
and `site/tests/test_website.py` checks that. Keep `site/assets/features.js` in step
with `design/sheprd-website/FEATURE-INVENTORY.md`, and describe availability
honestly: status badges and platform caveats are part of the page.

To update the social card, replace `site/assets/sheprd-social-preview.png` and
raise the `?v=` number on `og:image` and `twitter:image` in `site/index.html`.
Crawlers then treat it as a new image rather than using their cached copy.

Tests: `python3 -m unittest discover -s site/tests`.
