# T12: inherited site publication routes

Read-only source audit at `fd6a0809`. No website asset, remote site, Netlify
project, Cloudflare Worker, release artifact or owner setting was changed.
This records routes present in the checkout, not the current state of any
hosting account.

| Source route | Trigger and output in source | Spaceport consequence |
| --- | --- | --- |
| Root `netlify.toml` | Publishes root `site/`, but `build.ignore = "exit 0"` skips Git-triggered production, branch and preview builds. `site/tests/test_netlify_pause.py` checks all three contexts. `site/README.md` notes that build hooks and account settings are outside this source guard. | The root source is paused for automatic Netlify builds; this does not prove an external project is disconnected. |
| `website/netlify.toml` | A Netlify project with base `website` publishes `website/site/`. `build.ignore = "exit 1"` permits Git-triggered builds. `site/tests/test_website.py` requires that result and a public TestFlight link. | This separate source route can publish the inherited Sheprd site if an account connects this repository. The source cannot show whether the Spaceport fork is connected. The root pause does not apply to this base directory. |
| `website/herdrui/wrangler.jsonc` | Routes `herdrui-site` to `herdrui.dev` and `www.herdrui.dev`. `website/herdrui/README.md` gives an explicit Wrangler deploy command and says nothing deploys automatically. No current `.github/workflows` file invokes Wrangler or Netlify. | The checked-in Worker is a manual upstream-domain route; its deployed version and account ownership were not checked. |
| `.github/workflows/{release,promote}.yml` | Both use `workflow_dispatch`, guard jobs to `sheprd-labs/sheprd` on main and actor `penso`, and publish upstream release/tap assets only through those guarded jobs. The updater crate rejects `HERDR_RELEASE_VERSION` at compile time while Spaceport production identity remains unset. | These guards and the compile gate block the inherited package path in this fork. They do not gate a separate Netlify Git integration. |

`website/site/index.html`, `website/site/assets/features.js` and
`website/site/testflight/index.html` still describe a phone remote-control
web app, native iPhone/iPad client, widgets/Watch, and a public TestFlight
invite. T10 removed the corresponding desktop phone server and iOS/watch
source. Those statements belong to the inherited sheprd.org page, not a
qualified Spaceport site. `website/herdrui/public/index.html` still sends
visitors to sheprd.org; its `worker.js` serves mirrored installer and
`llms.txt` files at the old domain. Root `site/llms.txt` is also mirrored into
`website/site/` and the Worker. It advertises other removed Sheprd features
and upstream install/repository targets. `site/index.html` itself remains an
inherited Sheprd landing page even though its root Netlify build is paused.

The smallest safe Spaceport action before publication is to keep the current
release gate and root Netlify pause, and verify in the hosting accounts that
this fork is not connected to the upstream sheprd.org or herdrui.dev projects.
An owner-controlled Spaceport domain/project, production app and update IDs,
signing inputs and publication permission are pending decisions. Once those
exist, a distinct Spaceport site can be reviewed against the current product
and release artifacts before its deploy route is enabled. Changing
`website/netlify.toml` or its tests now could interrupt the separate Sheprd
site; changing only root `site/` would not remove the unpaused website route.
No ownership value is inferred from a checked-in URL or config.

`python3 -m unittest discover -s site/tests -p 'test_*.py' -q` passed 26
source/site tests; the Spaceport specification package check and
`git diff --check` passed. This audit did not
verify live Netlify settings, current pages, native installs, signed update
behavior or a published Spaceport artifact. G-RELEASE and G-PLATFORM remain
open; T12 stays blocked and A42–A44 remain not run.

## Fork source gate after this audit

Commit `5bd5c020` changed only this fork's `website/netlify.toml` to
`build.ignore = "exit 0"`, matching the paused root config. The revised site
policy test failed first against `exit 1`; the focused site suite then passed
20/20 and the full source/site suite passed 26/26. `website/README.md` now
labels the site as inherited and records the paused Git build. This guards
source-driven builds if a Netlify project were connected to the fork. It
does not disconnect or inspect any hosting account, prevent an authorized
manual deploy, change a live page, or supply a Spaceport domain or release
identity. No push or deploy ran. The historical route table above describes
the pre-gate source at `fd6a0809`; current source has both Netlify configs
paused. T12 and A42–A44 remain open.
