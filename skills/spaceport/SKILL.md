---
name: spaceport
description: "Use Spaceport's local client commands to inspect windows and chat, keep worktree notes, change settings, inspect processes, open or forward ports, mute notifications, keep awake, and wait for app events inside Herdr."
---
<!-- spaceport-managed-skill v1: Spaceport keeps this file up to date. Edit a copy under another name to keep your changes. -->

# Spaceport client commands

Use these commands for state and views owned by the running local Spaceport app.
Native chat supports Markdown, Mermaid, and the fixed `herdr-ui` control catalog.

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
| Open a dev server with the system URL handler, forwarding its selected SSH host's port first | `spaceport ports open 3000` |
| See the processes under a pane, with CPU and memory | `spaceport processes` |
| Start a chat with another agent in a new tab, shown as a native chat view | `spaceport chat new claude`, then `spaceport chat` for its status |
| Render existing Linux Codex/Claude conversations in their current panes | `spaceport config set chat.render_existing true` (needs `features.acp_chats`, on by default); `chat status --json` reports the session and input mode |
| List or focus the app's windows and its own tabs | `spaceport window`, `spaceport tab`, `spaceport tab focus ID` |
| Block until something happens in the app | `spaceport wait tab-closed ID --timeout 600`, `spaceport events --follow` |
| Read or change a Spaceport setting | `spaceport config get KEY`, `spaceport config set KEY VALUE` |
| Keep the machine awake during a long job | `spaceport caffeine on --for 2h` |
| Check for a Spaceport preview update (installed macOS preview only) | `spaceport run check-for-updates` |
| Ask the user to close this Herdr TUI workspace through the native confirmation | `spaceport run close_workspace` (opens the dialog; never confirms it) |
| Show recent Spaceport warnings to help diagnose a failed action | `spaceport run messages` (opens the window's memory-only history) |
| Check whether the app runs, and which windows and hosts it has | `spaceport status` |

Add `--json` after the command word for machine-readable output; errors then
come back with an `error` object containing `code` and `message`.
`spaceport events --follow` writes one JSON object per line.
Put options before note text or use `--` for literal text.
A command acts in the window that shows your workspace; it
exits 5 rather than act in another session's window.

| Exit | Meaning |
| --- | --- |
| 0 | Done |
| 1 | Refused or failed |
| 2 | Invalid arguments |
| 3 | Spaceport is not running |
| 4 | Nothing yet, or a wait condition was not met |
| 5 | Target not found |
| 6 | A dialog is open, or the app is busy |
| 7 | The app did not answer in time |
| 8 | Not available on this platform or app version |

The control socket stays local. Commands cannot reach it from an SSH host.
There are no browser or file-management commands; share a URL or path for the user to open.

Run `spaceport --help` to list every command, and `spaceport COMMAND --help`
for one.
