# T04/RM-PROVIDERS: Azure cloud provider cut

At `08f76a38`, Azure was a default cloud provider. Its API and local `az`
account path could create, list, connect to, stop, and delete VMs and Container
Apps sandboxes. An internal helper relayed sandbox exec streams into the legacy
client bridge. Settings, Add Device, the cloud registry, host load sampling,
and live website copy offered it. This cut removes those paths, their account
config, credentials, relay, feature flags, icon, and locale messages. It does
not contact Azure or remove any personal account, device, or file.

The old control `Shape` type was Azure-only dead code. At the base revision,
`git grep` found it only in `protocol/cloud.rs`, that module's JSON roundtrip
test, and `sheprd-remote::cloud::providers::Spec.shape`. `Spec` had no serde
derive, and no current control `Request` or `Response` used `Shape`. The
control protocol had already removed cloud request routes at version 18;
version 25's serialized envelope and bincode order do not change in this
cut. The standalone type and test were removed; 57 retained control protocol
tests pass.

The saved cloud catalog remains the one source for device rows. A mixed-row
test failed with `Invalid(Catalog)` while Azure was registered, then passed
after its registration was removed: an old `provider: "azure"` JSON value is
hidden from active devices, an active Container row is visible, and saving
retains the old value and its extra fields. A root `[azure]` account table is
unknown and inert. The strict `[cloud]` parser refuses `[cloud.azure]` and
does not rewrite the user's settings file. There is no compatibility shim or
personal cleanup; old `[cloud.azure]` settings need manual removal before
that file loads. The release review must state this clearly.

Container remains active, including its catalog, image picker, removal
warning, and idle checks. Manual SSH and Core machine onboarding are
unchanged. Azure DevOps and Azure OpenAI are separate still-live routes and
were retained. Azure Artifact Signing in the release workflow is unrelated.
The live website is inherited upstream Sheprd marketing; its Azure cloud
claim was removed but the site was not deployed. Design archives remain
inherited reference material.

Red checks: Azure source absence 2/2 failed on live routes; website absence
failed on Azure copy; the mixed catalog test failed with `Invalid(Catalog)`.
Green checks: Azure and Sprites source absence 4/4, website 25/25, mixed
catalog 1/1, retired GUI config 2/2, retained Container catalog 1/1,
unsynced removal 4/4, sandbox image picker 1/1, control protocol 57/57,
default and all-targets /
all-features app checks, i18n 4,456 messages in 245 files, formatting,
file-size, sorted-list, and diff checks. The macOS GUI link emitted its
existing `__eh_frame` warning; Cargo repeated the existing `block v0.1.6`
future-compatibility notice. No new code warnings remained.

Native GUI, provider endpoints, and installed service checks were not run.
The specification validator only checks schema and links, not app behavior.
T04 stays blocked on G-LINK while independent excluded-route cuts continue;
A30 stays `not_run`.
