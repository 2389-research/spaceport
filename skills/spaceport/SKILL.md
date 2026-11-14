---
name: spaceport
description: "Use Spaceport client commands for native chat, files, notes, settings, and window state inside Herdr. The browser commands have been removed; share a URL or file path for the user to open."
---
<!-- spaceport-managed-skill v1: Spaceport keeps this file up to date. Edit a copy under another name to keep your changes. -->

# Spaceport client commands

The browser command family has been removed. Share a web URL or
file path with the user for their system handler. Native chat supports Markdown,
Mermaid, and the fixed `herdr-ui` control catalog.

## More of Spaceport from the command line

The same executable drives the rest of Spaceport's own features. Herdr's own
`herdr` CLI still owns workspaces, terminal tabs, panes, agents, and
scrollback (`herdr pane read`): use `herdr` for those, and the commands below
for what only Spaceport shows or keeps.

| To | Run |
| --- | --- |
| Keep a note on this worktree (repository and branch) | `spaceport notes set waiting on CI` |
| Silence this worktree's notifications | `spaceport worktree mute` |
| Silence a whole host's notifications | `spaceport host mute studio` |
| Show the Automations section while Core has no job feed | `spaceport ui panel automations` |
| Open a dev server in a tab, forwarded when you run on an SSH host | `spaceport ports open 3000` |
| See the processes under a pane, with CPU and memory | `spaceport processes` |
| Start a chat with another agent in a new tab, shown as a native chat view | `spaceport chat new claude`, then `spaceport chat` for its status |
| Render existing Linux Codex/Claude conversations in their current panes | `spaceport config set chat.render_existing true` (needs `features.acp_chats`, on by default); `chat status --json` reports the session and input mode |
| List or focus the app's windows and its own tabs | `spaceport window`, `spaceport tab`, `spaceport tab focus ID` |
| Read app-observed session status and bounded attached text | `spaceport observation sessions --json`, then `observation conversation TARGET` or `observation terminal TARGET`; check coverage and truncation |
| Block until something happens in the app | `spaceport wait tab-closed ID --timeout 600`, `spaceport events --follow` |
| Read or change a Spaceport setting | `spaceport config get KEY`, `spaceport config set KEY VALUE` |
| Keep the machine awake during a long job | `spaceport caffeine on --for 2h` |
| Check for a Spaceport preview update (installed macOS preview only) | `spaceport run check-for-updates` |
| Check whether the app runs, its windows and hosts, and Ask Spaceport's state | `spaceport status --json` (read-only; submits no question) |

Add `--json` to any command for machine-readable output; errors then come
back as `{"error": {"code", "message"}}`. Exit statuses are the ones in the
table above. A command acts in the window that shows your workspace; it
exits 5 rather than act in another session's window.

Run `spaceport --help` to list every command, and `spaceport COMMAND --help`
for one.
