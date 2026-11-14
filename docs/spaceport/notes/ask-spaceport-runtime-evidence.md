# Ask Spaceport runtime evidence

Verified against primary documentation and source on November 13, 2026.
This research selects qualification candidates; it does not fully qualify an agent.
Source research was followed by isolated offline RPC inventory probes.
Those initial probes supplied no personal credentials and submitted no model prompt.
Later live tests use the separately approved scope below.

## Product constraint

Doctor Biz prefers installed Pi, Codex, and Claude Code.
Those programs should own their model routing and sign-in.
Spaceport must not add model endpoint or API-key settings for this feature.
Doctor Biz accepts qualifying Pi first.

## Candidate assessment

| Candidate | Observation-only boundary | Recovery evidence | Direction |
| --- | --- | --- | --- |
| Pi RPC | Exact tools plus disabled unowned extensions, MCP, skills, templates, and context discovery. Inspected source filters executable registration. | Session history and cancellation exist. Streaming text needs app-owned persistence. | Qualify first. |
| Claude Code | Built-ins can be removed and MCP restricted. Managed hooks and settings complicate isolation. Bare mode changes authentication. | Session resume and interrupt exist; pending-turn recovery needs proof. | Preserve as intended support; do not enable before qualification. |
| Codex app-server | Dynamic tools exist. Research did not establish a public blanket tool ceiling. Feature toggles alone do not prove exclusion. | Thread and turn identities help reconcile outcomes. Resume alone does not prove continuation. | Preserve as intended support; establish the tool boundary first. |
| OpenCode ACP | Version-specific permissions and merged configuration need a complete launch-profile proof. | Load/resume/cancel exist; loading does not restart inference by itself. | Outside the first qualification pass. |

### Pi

The inspected candidate is v1.5.2, not a promised supported version range.
Offline probes exercised both 0.87.1 and 1.5.2 through version-specific executable paths.
Version 0.87.1 does not support `--no-mcp`; its inspected built-in extension list has no MCP implementation.
Qualify each version separately; do not apply newer flags or upgrade it silently.
Use an owned observation extension and an exact executable tool allowlist.
Disable unowned extensions and MCP; an exact tools list alone can retain MCP tools.
The `--no-mcp` option appeared in 1.0.4.
Clear queued prompts before aborting; abort alone can leave queued work eligible to continue.
Pi persists completed messages, so received streaming text needs a separate durable projection.

Sources: [versioned CLI](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/docs/cli.md),
[executable registry](https://github.com/earendil-works/pi/blob/v1.5.2/packages/coding-agent/src/core/agent-session.ts#L1521),
[RPC commands](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/rpc-commands.md),
[changelog](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/CHANGELOG.md#104---2026-10-05).

### Offline inventory results

Four version/profile cases passed: baseline and restricted launches for both 0.87.1 and 1.5.2.
The baseline profiles exposed built-in tools and an owned denied sentinel.
The restricted profiles exposed exactly these inert fixture tools:

- `spaceport_list_sessions`
- `spaceport_read_conversation`
- `spaceport_read_terminal`

Attempting to activate the denied sentinel, shell, read, edit, and write tools did not restore them.
Each profile answered `get_state` and `get_commands` without a model prompt.
The probe supplied isolated HOME, configuration, session, temporary, and working directories.
Its constructed environment omitted inherited credentials.
It selected a catalog model only to open RPC; this does not select the production provider.
Each exact owned child was reaped after inspection.

Version 0.87.1 still advertised the bundled `llama` command outside the tool list.
Its inspected handler refuses RPC mode before TUI-only side effects.
Version 1.5.2 reported no commands in the tested isolated profile.
Native command dispatch remains a separate contract from executable tool activation.
Treat submitted questions as literal chat content rather than native slash commands.
Test that wire transformation before shipping the restricted role.

Primary 0.87.1 evidence: [tool registry](https://github.com/earendil-works/pi/blob/f07218c4d4bbc12bef056a7058c3dd49dfe41abe/packages/coding-agent/src/core/agent-session.ts#L3144),
[activation](https://github.com/earendil-works/pi/blob/f07218c4d4bbc12bef056a7058c3dd49dfe41abe/packages/coding-agent/src/core/agent-session.ts#L1279),
[built-in extensions](https://github.com/earendil-works/pi/blob/f07218c4d4bbc12bef056a7058c3dd49dfe41abe/packages/coding-agent/src/extensions/index.ts),
[RPC command guard](https://github.com/earendil-works/pi/blob/f07218c4d4bbc12bef056a7058c3dd49dfe41abe/packages/coding-agent/src/extensions/llama/index.ts#L183).

The ignored `target/ask-spaceport/` directory holds both probe sources and four named JSON reports.
Each report records the actual executable, version, sandbox, inventories, inspection results, and reaped child.
Independent review found no material correction to those four results.
These checks prove inventory and activation filtering, offline inspection, and direct-child cleanup.
They do not prove model tool dispatch, cancellation semantics, durable recovery, native integration, or the observation broker.
The probe did not independently trace network traffic.

### Claude Code

Removing built-in tools still leaves a lifecycle tool when MCP tools remain.
Strict MCP configuration does not override managed controls.
Bare mode avoids discovered customization but does not use subscription OAuth or keychain login.
That route conflicts with the intended existing-sign-in experience unless separately authorized.
Do not change managed policy, import credentials, or silently choose another authentication route.

Sources: [CLI controls](https://code.claude.com/docs/en/cli-reference),
[bare mode](https://code.claude.com/docs/en/headless#start-faster-with-bare-mode),
[managed MCP](https://code.claude.com/docs/en/managed-mcp),
[managed hooks](https://code.claude.com/docs/en/settings-reference#disableallhooks).

### Codex

Dynamic tool registration does not by itself remove built-in tools.
The inspected core tool policy is not established as a public app-server configuration contract.
An empty MCP map also does not prove inherited servers disappeared.
Qualify the actual advertised and executable inventory, including model-dependent tools.

Sources: [app-server contract](https://developers.openai.com/codex/app-server),
[thread wire](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/src/protocol/v2/thread.rs),
[tool policy](https://github.com/openai/codex/blob/main/codex-rs/ext/extension-api/src/tool_policy.rs),
[tool construction](https://github.com/openai/codex/blob/main/codex-rs/core/src/tools/spec_plan.rs#L1398),
[isolated upstream request](https://github.com/openai/codex/blob/main/codex-rs/tui/src/temporary_structured_request.rs).

### Mux

Doctor Biz proposed the team's Mux library as an alternative.
Inspected main commit: `984ef36338df13dff6836209b8f1677cf90f054e`, dated November 2, 2026.
Its shipped clients call provider APIs; its minimal example requires an OpenAI or Anthropic API key.
The caller supplies the LLM client, model, and credentials.
It has a filtered executable tool registry but no inspected installed-agent transport.
Its durable Resume expects suspended checkpoints, rather than arbitrary interrupted inference.
Using it now would add provider ownership and a Go integration without resolving turn recovery.
Keep it outside the first version under the existing-sign-in constraint.

Sources: [minimal example](https://github.com/2389-research/mux/blob/984ef36338df13dff6836209b8f1677cf90f054e/examples/minimal/main.go#L48),
[OpenAI client](https://github.com/2389-research/mux/blob/984ef36338df13dff6836209b8f1677cf90f054e/llm/openai.go#L27),
[agent wiring](https://github.com/2389-research/mux/blob/984ef36338df13dff6836209b8f1677cf90f054e/agent/agent.go#L39),
[tool filter](https://github.com/2389-research/mux/blob/984ef36338df13dff6836209b8f1677cf90f054e/tool/filter.go#L56),
[resume preconditions](https://github.com/2389-research/mux/blob/984ef36338df13dff6836209b8f1677cf90f054e/orchestrator/orchestrator.go#L812).

## Qualification requirements

- Inspect advertised tools and executable dispatch under hostile inherited configuration.
- Attempt denied shell, file, web, plugin, MCP, and subagent calls against synthetic sentinels.
- Repeat tool-boundary checks after model change, reload, and conversation resume.
- Preserve the agent's own configured provider and sign-in without copying credentials.
- Exercise cancellation before acceptance, during streaming, and during an observation read.
- Reconcile restart at submission, acceptance, streaming, tool execution, and completion boundaries.
- Prove single-owner continuation across repeated opens and multiple windows.
- Preserve explicit Stop and unknown outcomes without blind retries.
- Preserve literal question text without routing it through native slash-command dispatch.

The retained helper's ordinary launch defaults do not meet this restricted profile.
Relevant owners are `crates/sheprd-acp/src/pi.rs`, `claude.rs`, `codex.rs`, `harness.rs`, and `session/agent.rs`.

## Implemented helper evidence

The dedicated `ask` helper role uses the shipped extension and accepts only Pi 1.5.2.
Ordinary pane-chat launch defaults remain unchanged.
The real Pi loader accepted the production extension for a new session and an exact restored session.
Both offline cases exposed exactly the three observation tools and no native commands.
No provider prompt or personal credential was supplied.

The actual helper executable passed nine private fake-Pi runtime scenarios.
These cover literal questions, mutation refusal, startup proof, receipts, cancellation, and restart.
Two regressions hold acceptance until cancellation arrives.
They prove that a late actual start receives another cancellation before Stop or quit finishes.
Another regression reopens an unused Pi session before its session file exists.
History-bearing sessions still refuse a missing session file.

Final helper checks passed: 394 unit tests, one integration test, and 59 shared chat-protocol tests.
Control-protocol checks passed: 69 unit tests and four observation socket tests.
Strict Clippy passed for all four changed crates, all targets, with the integration-test feature.

The actual app worker also passed its ignored-abort shutdown regression.
Its private Pi fixture ignores abort and termination until the helper performs its owned cleanup.
The original three-second worker budget orphaned Pi; the seven-second budget reaps both owned children.
A second red/green regression prevents a full command queue from blocking app exit.
The complete worker finish budget includes startup, cancellation, and cleanup.

Independent backend review found no remaining material issue in the reviewed contracts.
Fixture results do not qualify real provider tool dispatch, cancellation, or recovery.

## Live provider qualification

Doctor Biz approved Pi's existing sign-in for isolated synthetic tests.
The first real turn invoked `spaceport_list_sessions` against an actual private Spaceport app.
The app used an explicit unavailable fixture socket and reported no connected sessions.
The completed tool output preserved that real empty coverage.
Pi finished the requested answer through its existing configured provider.
No model or authentication override was supplied.

Reopening that completed conversation preserved its request and user-message count.
It submitted no additional question and reaped the exact owned helper and app children.
The ignored fixture report retains both helper run identities and the initial harness cleanup error.
The helper consumes its quit marker; the corrected harness permits that marker to be absent.

Real streaming Stop passed against the configured provider.
The test waited for accepted inference and actual answer text before requesting Stop.
Reopening the stopped conversation preserved Stop without another user question.

Graceful shutdown during another real streamed answer confirmed an interrupted turn.
Reopening continued the same Pi session file and the same request identity.
The continuation added no duplicate user row.
The test then stopped that resumed turn and reaped its exact owned children.
The final private checkpoint retained three user rows: one completed question and two distinct lifecycle questions.
Its four recorded tool calls all used the allowed observation tools.
The checkpoint stayed mode 0600 inside a mode 0700 fixture directory.

The harness now waits for the attach transcript's live Idle status and checks each prompt acknowledgement.
A new helper ownership receipt alone does not establish loaded-session readiness.
The initial lifecycle harness failed before sending inference because it ignored that distinction.
The corrected harness and the private evidence audit pass.
The audit also reads Pi's own session file rather than relying only on app transcript rows.
It verifies exactly three protected original submissions and one continuation.
That excludes hidden completed or stopped replay even when a continuation hides its app user row.
Native source navigation and the full native provider flow remain under test.

### Final settlement correction

The live captures above predate the final settlement correction.
Pi 1.5.2 can emit `agent_end` while an automatic retry or compaction remains pending.
Its later `agent_settled` event confirms that no retry or queued continuation remains.
That event also reports whether the run was aborted.
The restricted bridge must retain its prompt until that authoritative event.
An earlier failed checkpoint does not prove Pi had stopped retrying.

The provider's organization concurrency limit currently blocks another native live run.
The correction passed deterministic bridge and real-helper tests without provider calls.
The helper suite passes 399 unit tests and one integration test.
All 13 helper scenarios pass, including recovered retries and cancellation during retry backoff.
A false run-abort flag preserves a cancelled final attempt instead of inventing successful completion.
Independent lifecycle review found no remaining material issue after that correction.
No provider, model, credential, or retry setting changes were made for that correction.

The final fixed-build native attempt completed both session-list and terminal-read tools.
The requested answer did not finish within the fixture's bounded wait.
The cause of that missing response is not established by the captured evidence.
Graceful cleanup reached authoritative cancellation and saved an interrupted request.
The owned helper released its lock before fixture cleanup.
No further provider attempt followed that timeout.
This validates live interruption on the corrected settlement path, but not final native source navigation.

### Coverage scope correction

The first live answer exposed a misleading broker label: disabled Core support appeared as a host.
Coverage now uses explicit connection, runtime, and capacity alternatives.
Only connection coverage carries a host and window identity.
Runtime coverage still reports Core as disabled or unsupported without reading Core state.
The actual broker wire regression failed before the fix and passes afterward.
The updated observation filter passes 36 tests, with two existing tests ignored.

### Rerun the live lifecycle checks

These macOS checks require explicit approval to use the installed agent's provider sign-in.
Select absolute app, helper, and Pi executables.
Use a private fixture directory; never point the test at a personal daemon.
The harness creates its own disconnected app and broker.
It preserves Pi's existing sign-in and model routing without reading credential files itself.

```sh
node crates/sheprd-acp/src/ask/tests/provider.mjs --help
node crates/sheprd-acp/src/ask/tests/provider.mjs --run-approved-provider-test \
  /absolute/path/to/spaceport /absolute/path/to/sheprd-acp /absolute/path/to/pi - lifecycle
node crates/sheprd-acp/src/ask/tests/provider_lifecycle_audit.mjs /absolute/private/fixture
```

The output names the private evidence directory.
To inspect only a previously completed fixture, replace `-` with that directory and select `completed`.
The harness refuses to treat unfinished state as a completed-only restart.
