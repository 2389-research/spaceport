# Fork, deletion and rebrand plan

Implements SP-001–SP-004 and SP-015–SP-016, SP-022–SP-026. Candidate roots are recorded in `plan/removals.json`; inspect the real transitive dependency graph before editing. The inventory is based on [S08, S09](SOURCES.md), not a complete source audit.

## 1. Preserve before removing

Fork at an explicit Sheprd commit. Record the origin without promising routine merges. Build and exercise an isolated baseline. Capture retained views, command behavior, emulator fixtures, permissions, terminal lifetime and relevant platform packaging. Record any pre-existing failures instead of laundering them into successful tests.

Separate behavioral deletion from mechanical renaming. First establish Core-only end-to-end paths and typed view readers; then remove obsolete reachability. A directory named `sheprd-client` may mix old protocol code with useful helpers. A directory named `sheprd-protocol` still supplies Core rendering/input types. Extract retained pure primitives before deleting the legacy adapter. Preserve Core protocol generation 1. [S02, S09, S11]

## 2. Deletion units

A deletion unit is a complete unwanted behavior: UI entry points, command catalog entries, settings/schema fields, background workers, source modules, assets/translations, dependencies, docs, fixtures and package/service integration. Do not count code as removed while it remains reachable from a hidden CLI, migration path or worker.

| Family | Candidate code roots | Treatment |
|---|---|---|
| Git/diffs/IDE/indexing | `sheprd-vcs`, relevant GPUI review/code/editor modules | Delete specialized behavior, retain plain chat/terminal text |
| GitHub and issue dispatch | `sheprd-github`, `sheprd-orchestrator` | Delete accounts/PR/issues/security/Beads dispatch; extract generic cache/file primitives only if independently necessary |
| Cloud and Teleport | `sheprd-remote` and GPUI provider UI | Remove cloud features, provider credentials/setup and migration; audit shared helpers before deleting whole crate |
| Automations/mods | `sheprd-automate`, `sheprd-automation`, `sheprd-luau`, `sheprd-mods` | Delete schedulers/execution/editing/runtime; retain only proven passive display types needed by an external feed |
| Embedded pages/annotations | GPUI browser module and webview-specific dependencies | Delete browser, IDE webviews and annotation handlers; retain ordinary system URL/file opening |
| Mobile service/push/pairing | `sheprd-remote-control`, `sheprd-apns`, `sheprd-push-relay`, `sheprd-nearby`, `ios/`, platform nearby helpers | Delete from the desktop fork/build graph; extract shared pure session/chat types before removal |
| Legacy Herdr 1 | Legacy client/snapshot/bridge paths | Delete after Core-only replacement; do not delete shared types or Core wire generation 1 |
| Provider usage/config | provider probes/sign-in inside `sheprd-usage`, UI model/provider settings | Delete client-owned collection/config; retain reported values from supported runtime/agent sources |
| Terminal feedback | terminal annotation/send-selection-to-agent paths | Delete; normal selection/copy/paste remains |

Negative requirements apply to **product code**, not to the engineering repository's ordinary Git history, GitHub Actions CI or tests mentioning removed names. Likewise, absence checks must not reject a user transcript containing `git diff`, a Rust dependency with an incidental name, or legally required attribution.

## 3. Retained code candidates

Reuse GPUI, terminal emulator, Core adapter, pure session and chat reducers, control-protocol/command catalog, diagnostics, i18n, updater and Windows launcher. Rename their package identities deliberately after behavior is stable. Avoid a top-to-bottom architecture rewrite.

`sheprd-acp` is a special case: retain the minimal verified host-side agent adapter contract necessary for native chat, not its desktop-owned resume/deployment or model/provider-management workflows. Its process belongs to Herdr. A compile-time dependency is not permission to keep excluded behavior.

Credentials are another mixed case: delete provider/account uses; retain only generic OS/private-endpoint primitives that a kept feature demonstrably needs. Do not broadly migrate credential stores. Upstream “report-only” automation features are not automatically compatible with a Core-owned data source.

## 4. Reachability and dependency proof

For each deletion record: inventory call sites and entry points; identify shared survivors; make the smallest extraction; remove behavior; update command/config/assets/docs; regenerate lockfile with the chosen toolchain; run retained-feature tests plus negative reachability tests. Verify the release dependency graph on all targets and feature combinations actually shipped.

Searches are supporting evidence, not sole proof. A string remaining in attribution can be correct; a worker without UI can still violate scope. Check build features, packaged binaries, launch agents/services, updater helpers and platform-only source. Use fake provider credentials/servers to prove removed components are never contacted, without real network side effects.

Delete old tests only when their tested product behavior is intentionally removed. Keep renderer, identity, permission and lifecycle regression fixtures even when their old file location disappears. Add representative tests proving excluded commands/configuration are rejected and obsolete background services are not installed.

## 5. Rebrand domains, not raw strings

**Product identities:** Spaceport display names, executables, app/package IDs, bundle resources, application state, local control endpoint, installer metadata, desktop entries, URL handlers, help, website references and update endpoints.

**Compatibility identities:** Herdr RPC method names/fields, `hc` invocation, protocol constants, agent ecosystem variables, existing host-side attach contract identifiers. Change these only with explicit versioned adapter compatibility evidence. Do not replace every occurrence of `herdr` or `acp`.

**Attribution:** Keep applicable license and copyright notices and document the fork origin. Do not claim trademark/domain availability or upstream endorsement.

An engineering placeholder application ID is allowed only in isolated development builds. Shipping owner-controlled IDs, signing accounts/keys, update channel and icons are release inputs. Unknown values are hard release gates, never silently inherited upstream identifiers. A signed Spaceport update must never install Sheprd or Herdr.

## 6. User state migration

Default to independent Spaceport state. Do not reuse the upstream state directory, mobile App Group, service names, keychain access group or updater identity. If preference migration is implemented, make it an explicit copy from an existing app, validate compatible fields, and exclude GitHub/cloud tokens, runtime recipes and removed settings. Leave the original untouched. No migration is required just to complete the code-removal task.

## 7. Completion evidence

Provide before/after dependency and packaged-binary inventories, retained scenario results, representative screenshots, removed command/settings checks, rebrand identity report and an unresolved-gate list. File count or binary-size reduction alone is not acceptance. A fork that compiles but quietly loses native chat, forwarding or terminal persistence is not finished.
