# T03 typed Core group key: route map and size gate

Read-only source audit at `42664731`, using the T03 task packet and the
[document model](t03-workspace-document.md) and
[Core group UI](t03-workspace-ui.md) evidence. No app code, Core process, or
saved user state changed. This note stops before the requested 450-line
architecture gate; it does not claim a typed key was implemented.

## Current authority and keys

The `spaceport` Core document has UUID workspace IDs and ordered members
identified by `{core_id,terminal_id}` (`core_terminals/workspace_document.rs:27-55,
126-145`). The current GUI observer projects the local named document and
its Core identity (`core_terminals/observation.rs:195-203`); group selection
keeps only a UUID in window state (`core_terminals/workspace_ui.rs:32,184-196`).
Core `doc.get`/`doc.set` accept a document name and optional `on` machine
target (`sheprd-core-client/src/connection.rs:309-329`). The private remote
document test refetches with the registered machine's **permanent ID** after
renaming it (`sheprd-core-client/src/tests/live/remote_document.rs:34-46,
96-104`). A remote notice can carry a machine name, so that notice is an
invalidation, not a stable saved tab key (`backend.rs:109-115`). A future
Core group key must include the hub Core identity, local versus permanent
machine ID, document name, and group UUID. Group labels and machine aliases
are display text, not identity. Two machines may each have the same label and
group UUID without sharing a tab or layout.

Today `browser::Store::Tab` saves flat `scope: Scope(String)` and
`workspace_id: String` fields (`browser/store.rs:69,138-160`). The same pair
keys `Layouts`, including saved `scope` and `workspace` fields
(`browser/layouts.rs:15-26`). `Browser` holds one layout map and one saved
layout map keyed by the pair (`browser/view.rs:27-35`); its hidden-tab map and
closed-tab history use it too (`browser/hidden.rs:1-19`,
`browser/closed.rs:145-176`). The saved `Pick::Page(TabId)` selects a tab
within the layout (`browser/groups.rs:58-61,89-114`). Store tab IDs are
app-global and must not be reused after an old tab is discarded
(`browser/store.rs:274-294`).

The Core member click currently calls `browser_key()` and opens or reuses a
Core terminal tab under that **legacy focused workspace**
(`core_terminals/workspace_ui.rs:269-351`). Plain Core terminal creation and
view sync also read that key (`core_terminals.rs:365-384`,
`core_terminals/tabs.rs:18-41`). `browser_key()` itself reads the legacy
snapshot's focused workspace (`browser/view.rs:178-181`). The group strip,
layout, pane views, and sidebar all filter by it
(`browser/groups_view.rs:70-105,323-352,691-714,764-824`,
`browser/view/render.rs:36-44`, `core_terminals/sidebar.rs:18-38`). Adding
only a typed Store key would make a Core tab disappear from the visible
strip, group selection, or pane view. Adding a second Core Store/Layout would
duplicate the window's view authority.

## Minimum coherent change

Use one discriminated key, `Legacy { scope, workspace_id }` or
`Core { hub_core_id, machine_id: None | Some(permanent_id), document_name,
group_id: UUID }`, throughout the existing Store, Layouts, Browser, and
window selection path. The Core branch must be built from fresh observer
identity, and remote keys must wait for a proved permanent machine mapping.
Old flat tab and layout records must still deserialize as legacy records.
The old Core terminal tabs have no saved group provenance; keep them under
their old legacy workspace until an explicit group action places them. Do
not infer a Core group from a matching label, UUID, terminal address, or
machine alias. New Core records need an explicit tagged saved shape and
bounded validation; their tab IDs and saved group picks must remain stable.

The necessary diff is estimated at **560–760 changed lines**, before native
fixes: 120–160 for typed key and saved tab parsing, 90–130 for Layouts and
window maps, 140–190 for active-key strip/pane/sidebar/close behavior,
70–100 for the Core member/open route, and 140–180 for regressions. This is
an estimate from the call-site map, not completed implementation. It exceeds
the 450-line gate for this delegated slice.

The red/green plan needs (1) two Core machines with colliding labels and
group UUIDs but distinct permanent IDs, each retaining its own Store tab and
saved layout; (2) a legacy workspace with the same string ID remaining
separate; (3) round-trip of old flat tabs/layouts and new tagged Core records,
including stable, non-reused tab IDs and `Pick::Page` after restore; and (4)
a headless group click whose selected Core tab stays visible in the strip and
pane while switching away and back. A private Core scenario should then
exercise local exact identity; remote UI needs the permanent-ID mapping and
fresh-document watch first. The existing headless membership test and
private Core document test cover different seams, so they do not prove this
saved-tab selection path. Native narrow/focus/IME and cross-platform results
remain open. T03 stays `in_progress`; A06/A09/A24/A53 stay `not_run`.
