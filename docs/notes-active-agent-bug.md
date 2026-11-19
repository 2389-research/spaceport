# Agents that "pop up and show active" when they are not (crew 57b2)

Bug report, verbatim: *"bug in how we determine active agent."* / *"sometimes
agents will just pop up and show active. they are not"* (P1, no further
detail). This note is the root-cause analysis and the fix.

## Root cause

Spaceport does not decide that an agent is active. It paints
`ClientShellAgent::agent_status` from the daemon's snapshot, unchanged. For the
agent the crew runs (`pi`), the daemon does not know the state either: Herdr
infers it by scraping the pane's text with rules from a downloaded *agent
detection manifest*, and that manifest's `working` rules match ordinary pane
text, including tool output. A pane that is idle (or a pane whose agent never
started) is then reported `working`, and every client — Spaceport, Herdr's own
TUI — shows a working dot for it.

So the answer to "how we determine active agent" is: *we* do not; the daemon
guesses, and the guess is wrong. There is no code in this repository that can
be corrected to make the dot right without breaking the rule that every client
shows the same dot (`AGENTS.md`: "Activity indicators come from the daemon's
snapshot, not guessed terminal output or a client-local seen ledger").

That one word is not cosmetic: the same status decides whether closing a tab
asks first (`crates/sheprd-gpui/src/close_modal.rs:114`), keeps the machine
awake (`crates/sheprd-gpui/src/caffeine/work.rs:19`), and pulses Overview's
icon (`crates/sheprd-gpui/src/overview.rs:239`). A false `working` is also an
extra confirmation dialog and a Mac that will not sleep.

## How Spaceport gets the dot (verified in code, no heuristic anywhere)

| Step | Where |
| --- | --- |
| Wire status, one word per agent | `crates/sheprd-protocol/src/wire.rs:419` (`ClientShellAgent::agent_status`), decoded by `deserialize_client_shell_agent_status` at `wire.rs:297` |
| Transport only; nothing rewrites it | `crates/sheprd-client/src/session.rs:670` (snapshot decode) → `ClientEvent::Snapshot` |
| Committed snapshot the sidebar reads | `crates/sheprd-gpui/src/state/projection.rs`, `state.rs:483` (`agent_status`) |
| Agents panel rows | `crates/sheprd-gpui/src/sidebar/agents.rs:72` (`status_priority`), `:390` (`status_indicator`), `crates/sheprd-gpui/src/sidebar/render/agent_rows.rs` |
| The spinning "working" mark | `crates/sheprd-gpui/src/sidebar/agents.rs:451` (`working_orbit`), `icon_mark` |
| Overview sections and the bar's pulse | `crates/sheprd-gpui/src/overview/model.rs:72` (`State::of`), `overview.rs:239` (`overview_working`) |
| Keep-awake and Dock badge, same word | `crates/sheprd-gpui/src/caffeine/work.rs:19`, `app_badge.rs:106` |

The only two places in the tree that *mint* an agent status are programs
reporting themselves: `sheprd-acp` posts `pane.report_agent`
(`crates/sheprd-acp/src/herdr.rs:261`) from its own transcript, and writes
OSC 7501 program status to its terminal
(`crates/sheprd-acp/src/tui.rs:99`, `tui/view.rs:453`). Neither is involved in
the reported symptom: `pi` panes are not Sheprd chats.

The existing regression test for the contract is
`crates/sheprd-gpui/src/state/tests/agent_status.rs`
(`daemon_status_reaches_the_sidebar_unchanged`): every status the daemon
reports reaches the dot verbatim, across a boot change and a reconnect.

## How the daemon decides it (measured on this machine)

Herdr 0.9.5 (`~/.local/bin/herdr`) ships per-agent *detection manifests* and
refreshes them from `https://herdr.dev/agent-detection/index.toml`; the live
copy for `pi` is
`~/.local/state/herdr/agent-detection/remote/pi.toml`, version
`2026.10.01.1`. Its only two rules are:

```toml
[[rules]]
id = "working_literal"
state = "working"
region = "whole_recent"          # last ~4 KiB of the pane, tool output included
any = [
  { contains = ["Working..."] },
  { line_regex = ['^[⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏] Working$'] },
]

[[rules]]
id = "working_border"
state = "working"
region = "bottom_non_empty_lines(12)"
any = [
  { line_regex = ['^── [⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏] Working ─+$'] },
  { line_regex = ['^[⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏] Working$'] },
]
```

There is no `idle` rule; when nothing matches, Herdr falls back to `idle`
(`fallback_reason: default_known_agent_idle_fallback`).

`working_literal` is a plain substring test over the whole recent buffer, so
any text the agent printed — a build log, a test name, a quoted message, the
user's own prompt — can flip the pane to `working`. Reproduced deterministically
without touching any live pane, using Herdr's own rule evaluator:

```console
$ printf '⏺ Bash(npm test)\n  ⎿ Working...\n\n❯ \n' > b_tool_output.txt
$ herdr agent explain --file b_tool_output.txt --agent pi --format json
  state=working rule=working_literal      # an idle pane's tool output
$ printf '⏺ Read(src/world.js)\n  ⎿ Took 0.1s\n\n❯ \n' > d_idle.txt
$ herdr agent explain --file d_idle.txt --agent pi --format json
  state=idle    rule=None                  # control
```

An idle pane emits no further output, so matching text stays in the recent
buffer and the false `working` persists — the "stuck active" the report
describes. Because the rules are downloaded data rather than compiled-in
heuristics, the exact offender can also change between manifest versions.

Live evidence that the word is a guess in both directions, taken from this
machine while the bug was investigated:

- `herdr agent explain wJ:p1` — a pane genuinely mid-turn: `state=working`,
  `matched_rule=working_border`. (Control.)
- The pane running this very investigation, while it streamed tool calls and
  had two subagents open, reported `agent_status=idle` with `state_change_seq`
  frozen for minutes (`fallback_reason: default_known_agent_idle_fallback`).
  The daemon says *idle* for an agent that is demonstrably working, which is
  the same heuristic failing the other way.
- `~/.config/herdr/herdr-server.log` shows Herdr re-detecting `pi` in panes
  repeatedly (`herdr::pane: agent changed pane=… previous_agent=Some(Pi)
  agent=None … agent=Some(Pi) process=pi`), i.e. the detection pass that
  assigns the state runs on every foreground-process change.

## The fix

Nothing in Spaceport's rendering changes: a per-client "actually active" flag
is exactly what `AGENTS.md` forbids, and it would make Spaceport's dot disagree
with Herdr's own client for the same pane.

The fix is to stop making the daemon infer `pi`'s state from text, in this
order:

1. **On this machine, and for every agent the crew runs:** install the agent's
   Herdr integration, so the agent reports its own state instead of being
   screen-scraped.

   ```console
   $ herdr integration install pi        # writes ~/.pi/agent/extensions/herdr-agent-state.ts
   $ herdr integration status            # verified "not installed" here before the fix
   ```

   The Spaceport app offers the same thing without a shell:
   **Settings → Integrations → pi → Install** (`crates/sheprd-gpui/src/integrations.rs`,
   `install_integration`, which posts the daemon's `integration.install`).
   The integration is Herdr's own mechanism for exactly this: it posts
   `pane.report_agent` from the agent's lifecycle hooks, as `sheprd-acp` already
   does for Sheprd's chats, and Herdr then treats those reports as the pane's
   authority (`herdr agent explain` reports the field
   `full_lifecycle_hook_authority`). It was deliberately not installed while
   writing this note — it writes into the user's `~/.pi` and takes effect in new
   agent sessions, which is the owner's call, not a subagent's.

   Measured caveat on this machine: the daemon answers `integration.list` with
   `{"target": "pi", "available": false, "state": "not_installed"}`, because
   `pi` is not on the daemon's `PATH` (it lives behind a `mise` shim). Spaceport
   renders that row as unavailable (`integrations-unavailable`) and offers no
   Install button, so the shell command above, run from a login shell, is the
   route here. Making the daemon see the user's login `PATH` is a Herdr/server
   concern, not this client's.

2. **Upstream (Herdr owns this; we never patch Herdr):** report the misfire and
   tighten `pi.toml` — drop `working_literal`'s `contains = ["Working..."]`
   over `whole_recent`, keep only footer-anchored rules, and add an `idle` rule
   so a known agent's resting state is detected rather than defaulted. The
   manifests are remote data, so a manifest release fixes existing
   installations without a Herdr build.

3. **Spaceport, if a client-side change is ever wanted:** show *which* source
   answered — an agent with no `agent_session` on its pane has no lifecycle
   reporter, so its dot comes from screen detection. That is a product decision
   (a new indicator, not a correction to the dot) and is deliberately not made
   here.

## Rejected fixes

- Derive the dot from pane text in the client: reintroduces exactly the
  heuristic that caused the bug, in a second place, and lets Spaceport and
  Herdr's client disagree.
- Keep a client-local "seen"/acknowledged ledger so a `working` an idle-looking
  pane stays quiet: forbidden by `AGENTS.md`; every window would then show a
  different dot.
- Hide agents whose pane has no `agent_session`: hides real screen-detected
  agents (every `pi` pane today), and invents a policy the daemon does not
  have.
- Map `AgentStatus::Unknown` to `Idle` (or silence it) so a guess looks
  calmer: `crates/sheprd-gpui/src/sidebar/tokens.rs:114` already maps unknown
  to idle for configured tokens; doing it to the painted status would hide the
  daemon's own "I do not know".

## Verification

- The dot's contract is pinned by
  `crates/sheprd-gpui/src/state/tests/agent_status.rs`, which this note relies
  on rather than duplicates: no client-side code path can rewrite a reported
  status.
- Two of the candidate causes this report could have had are now pinned in
  `crates/sheprd-gpui/src/sidebar/tests/agent_activity.rs`, through the same
  `panel_agents` the panel paints from:

  - `a_working_dot_does_not_outlive_the_agent_the_daemon_listed` — the daemon
    dropping an agent drops its row and its dot; no stale `working` survives an
    exit.
  - `a_restarted_daemon_paints_the_status_it_reports_now` — a new boot reusing
    a pane id paints the new status, so an epoch mismatch cannot re-show an old
    active state.

  Both pass against the code as it stands: they record that these paths are
  *not* the bug, so a future change cannot make them one, and they are not a
  behaviour fix on their own.
- The daemon-side claim above is reproducible with the two `herdr agent
  explain --file` commands shown; both were run while writing this note.
- Re-diagnosing a suspect pane, without changing anything:

  ```console
  $ herdr api snapshot | jq -r '.result.snapshot.agents[] | [.pane_id, .agent, .agent_status, .state_change_seq] | @tsv'
  $ herdr agent explain <pane> --format json     # which rule, and its region
  $ herdr pane read <pane> --source visible --lines 200 | tail -6   # what the agent itself shows
  ```

  A pane whose own footer names no running turn while `agent_status` is
  `working` is this bug. `herdr integration status` says whether the pane's
  agent reports for itself yet.

## One Spaceport-owned caveat, out of scope

`program_status` (`crates/sheprd-acp/src/tui/view.rs:453`) reports a chat that
is still `Status::Starting` as `working` over OSC 7501, so a Sheprd chat's
pane dot spins while the agent boots. That is our own program reporting itself
(not a guess), it is brief, and changing it changes what users see for a chat
that *is* starting, so it is recorded here rather than folded into this fix.
