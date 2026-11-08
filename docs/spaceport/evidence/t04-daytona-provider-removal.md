# T04/RM-PROVIDERS: Daytona provider cut

At `6f10ce62`, Daytona creation through dispatch was already absent, but the
Settings account card still read and saved API keys, the provider registry
still connected and deleted saved sandboxes, the image support still built and
listed snapshots, and Add Device still suggested Daytona from config or the
environment. This cut removes those routes, the provider implementation and
feature flags, its icon and locale messages, and current user documentation.
The shared image catalog, Container image card, credential store, cloud socket
and other providers remain. The separate Core machine catalog and Add Device
gate are unchanged. Generic `host` control targeting is unchanged; no Daytona
endpoint is offered by the catalog.

The saved cloud-device catalog already kept unknown provider entries. A new
test writes an old `provider: "daytona"` device, requires it to be hidden,
adds an active provider device, and compares the old structured value after
the write. The shared sandbox-image catalog had no such path: deleting the
Daytona enum would make an old image row fail the whole file and hide active
Container images. A red test reproduced two visible images where only the
Container image should appear. The catalog now reads unknown image providers
as inert JSON values, rejects malformed known-provider rows, hides foreign
rows from typed callers, and keeps them when a Container image is recorded or
forgotten. The test compares the old row's structured value after both writes.
This preserves saved file data; it does not restore Daytona behavior. No
personal catalog or credential was read or changed.

Old root `[daytona]` settings become unknown and inert. An old
`[cloud.daytona]` setup table is unsupported by the strict `[cloud]` parser
and makes config load fail. A new test requires that startup reject it without
rewriting the file. No parser shim, migration or personal cleanup is included.
This startup limit remains a release review item.

The live Sheprd website is upstream marketing, not Spaceport release proof.
Its published `website/site` page and feature directory no longer advertise
Daytona; a website regression failed first, then the 19 site tests passed.
Design archives and the separate root-site demo still name Daytona and other
excluded upstream features. No website was deployed.

The source and asset absence test failed before implementation on live
provider routes and assets, then passed 2/2. The two focused remote catalog
tests passed 2/2 with Boat, Container and Coder features. `just check-i18n`
passed (4,979 messages in 261 files); formatter, file-size, sorted-list and
diff checks passed. The published site tests passed 19/19 and `node --check`
accepted its edited feature script. Package `spec.py check` passed with JSON
Schema, and the package's 21 tool tests passed. A focused no-default GUI test
build was stopped before link at 9.9 GiB free to keep the 8 GiB disk floor.
GUI config tests, default/all-feature app checks, native UI and
installed-service checks remain unrun on this branch for merged verification.
T04 remains blocked on G-LINK;
A30 remains `not_run` while other excluded provider and runtime routes exist.

## Integrated branch verification

After merging as `b7e6db6e`, default and all-feature production app checks
passed with `--locked` and no new code warnings; Cargo repeated only its
existing `block v0.1.6` notice. The retained device and image catalog suite
passed 15/15 with Boat, Container and Coder features, including foreign
Daytona preservation, active Container retention, and corrupt-file refusal.
The merged GUI config cases passed 2/2; the Core machine catalog filter
passed 5/5 and remote document filter passed 6/6 (one separate opt-in case
ignored). The source/asset absence test passed 2/2 and the published-site
suite passed 19/19. The first site invocation used Python's built-in `site`
module path and failed before discovery; the corrected
`python3 -m unittest discover -s site/tests -p 'test_website.py' -v` passed.
Format, file-size, sorted and i18n checks passed; i18n validated 4,973
messages in 261 files. The GUI test link repeated the known macOS
compact-unwind warning. No native cloud/Core UI, installed service, or
published website was exercised by these merged checks.
