# Decision reconciliation and architectural corrections

This document resolves contradictory assistant paraphrases and false implementation assumptions without reopening the user's product preferences. The chronological ledger remains intact in `plan/decisions.json`.

## R1 — Preserve by default, not redesign by questionnaire

The repeated “keep existing” choices establish the baseline. Existing desktop visual hierarchy, interaction patterns and customization survive unless expressly removed. The new agent ergonomics primarily improve inspection, control correctness, module boundaries and implementation evidence—not a new navigation design.

D49 (“No change”) limits the earlier broad activity-history language. Keep the existing timeline and search its available data. Do not add a global event database, cross-host history service or a new activity-center architecture.

## R2 — Herdr Core is remote-capable

The earlier local-only claim conflated Sheprd's Core integration with Core itself. The inspected Core documents say the local Core owns remote connections, terminal/event aggregation and retry. Spaceport consumes that path. Preserve the host-manager UI, not its old independent multi-host backend. [S01–S03]

D48/D56 therefore qualify D40. A saved machine is Core state; a favorite/color/selection is client presentation. Do not keep two authoritative host catalogs.

## R3 — Workspaces need presentation semantics

The earlier claim that Core owns the old daemon's full workspace/agent model was too broad. Sheprd's own session-model document says Core owns terminals and describes unfinished client session-model work. Core separately exposes opaque versioned documents. [S02, S04]

The engineering resolution is a Spaceport-owned **view schema** persisted through Core's document API. It is not a scheduler, execution database or invented `workspace.create` endpoint. Create/rename/archive/delete grouping stays supported; process termination is a different explicit operation. This preserves the user's “client, not replacement runtime” boundary while making the workspace UI implementable.

Writing Core-backed metadata does not make Core interpret that metadata. Documentation and inspection must say which layer owns its semantics. Do not claim interoperability with another client's private workspace schema or silently write `herdr-app`.

## R4 — Native chat is a host-side application contract

Rich chat is not supplied by Alacritty or by Core's raw terminal stream. Sheprd has a host-side adapter, and its Core-specific native attach path is documented as local. [S05, S06, S11]

Preserve the pure renderer/protocol and necessary existing host-side adapter, with Core owning its execution. Remove desktop-owned restart/resume/conversion/deployment policies. Local support does not prove remote parity. A missing Core-mediated remote chat route is an explicit external dependency, not license to restore the deleted remote-control server.

## R5 — The client is thin, not stateless

Client-owned themes, window layouts, input composition, terminal emulation, local CLI IPC and notifications are legitimate. “No independent workspace state” means no competing execution authority or offline operational database, not the impossibility of a view model. Persistent selection/preferences are compatible with online-only content access.

Last-window close quits Spaceport, so desktop notifications stop with it. No background notification daemon is added to satisfy earlier generic “full notifications” wording. Mobile push is out of scope.

## R6 — Agent controls versus agent configuration

D27 removes client-owned provider/model configuration, and D32 means use supported runtime/agent interaction. Keep explicit launch/template selection through Core and dynamically supported prompts/permissions. Do not add provider account screens, credential probes, model selectors or execution policy stores. An agent's terminal may naturally accept its own commands; Spaceport does not censor or independently implement them.

Read-only automation/usage visibility cannot depend on keeping the removed Sheprd scheduler/provider collector. Show only supported source data; record missing feeds as a capability gap. Do not manufacture zero usage or an empty job list.

## R7 — Delete behavior, not every shared type

Herdr 2's current wire protocol is generation 1. `sheprd-protocol` types are also used by the Core renderer. The removed feature is the legacy Herdr 1 backend. Extract necessary pure rendering/input types before deleting old snapshot/framing/transport behavior. [S02, S09, S11]

Deleting a mobile server does not delete the desktop CLI's private local control endpoint. Deleting Git UI does not prohibit using Git to build the fork or rendering a plain textual patch in chat.

## R8 — Retained features can reveal external gaps

Core's inspected method table has no dedicated forwarding or rich-chat API. General file APIs do not by themselves prove binary transfer parity. [S03]

Keep those requirements and prove their routes. Missing support blocks the affected completion claim. Do not silently drop the feature, build an alternate backend, call terminal fallback full parity or turn “unsupported” into a permanent hidden feature flag. Continue independent fork work, with minimal external contract requests documented for the backend owner.

## R9 — Rebranding is separate from redesign and compatibility

Full Spaceport identity is required; wholesale layout redesign is not. New names/icons/app IDs coexist with retained interaction design. Preserve required attribution and external protocol constants. Unknown production signing/ownership values are release gates, not defaults borrowed from upstream.
