---
name: herdr-gpui-browser
description: "Use Sheprd client commands for native chat, files, notes, settings, and window state inside Herdr. An SSH host can still request a page through its temporary shim; the local browser open command has been removed."
---
<!-- herdr-gpui-managed-skill v1: Sheprd keeps this file up to date. Edit a copy under another name to keep your changes. -->

# Sheprd client commands

The local `sheprd browser open` command has been removed. Share a web URL or
file path with the user for their system handler. Native chat supports Markdown,
Mermaid, and the fixed `herdr-ui` control catalog.

An SSH host with the installed shim can still request a web page through
Herdr workspace metadata with `sheprd browser open URL`. That path is
transitional while embedded browser support is removed.

## More of Sheprd from the command line

The same executable drives the rest of Sheprd's own features. Herdr's own
`herdr` CLI still owns workspaces, terminal tabs, panes, agents, and
scrollback (`herdr pane read`): use `herdr` for those, and the commands below
for what only Sheprd shows or keeps.

| To | Run |
| --- | --- |
| Show a file at a line, read-only | `sheprd code open src/main.rs:42` |
| Open the review of your uncommitted changes | `sheprd review` |
| Open px0 to browse, edit, or review your local checkout (macOS/Windows) | `sheprd px0 open` |
| Get notes the user sent this pane (pages, reviews, terminals) | `sheprd feedback --wait 600` |
| Keep a note on this worktree (repository and branch) | `sheprd notes set waiting on CI` |
| Silence this worktree's notifications | `sheprd worktree mute` |
| Silence a whole host's notifications | `sheprd host mute studio` |
| Save the working tree before a risky change, and compare later | `sheprd checkpoint create before refactor`, `sheprd checkpoint diff ID` |
| Your branch's pull request, its checks, and (with your workspace focused) its comments | `sheprd pr`, `sheprd pr checks`, `sheprd pr comments` |
| Diagnose missing GitHub details: last reported GraphQL quota and rate-limit retry delays | `sheprd pr quota --json` |
| List the vulnerability reports outsiders filed and the open Dependabot alerts of your repositories, most severe first | `sheprd security --json` |
| Show the user an alert's or a vulnerability report's page in Sheprd | `sheprd security open OWNER/NAME 42` or `... GHSA-xxxx-xxxx-xxxx` |
| Show the Automations section while Core has no job feed | `sheprd ui panel automations` |
| Open a dev server in a tab, forwarded when you run on an SSH host | `sheprd ports open 3000` |
| See the processes under a pane, with CPU and memory | `sheprd processes` |
| Start a chat with another agent in a new tab, shown as a native chat view | `sheprd chat new claude`, then `sheprd chat` for its status |
| Move a terminal agent into a native chat, keeping its conversation (an agent that is idle, in another pane) | `sheprd chat convert --pane ID`, then `sheprd chat` for its status |
| Render existing Linux Codex/Claude conversations in their current panes | `sheprd config set chat.render_existing true` (needs `features.acp_chats`, on by default); `chat status --json` reports the session and input mode |
| List or focus the app's windows and its own tabs | `sheprd window`, `sheprd tab`, `sheprd tab focus ID` |
| Block until something happens in the app | `sheprd wait tab-closed ID --timeout 600`, `sheprd events --follow` |
| Read or change a Sheprd setting | `sheprd config get KEY`, `sheprd config set KEY VALUE` |
| Install agents or rerun the setup script on a saved cloud machine | `sheprd config set cloud.agents '["claude"]'`, `sheprd cloud setup ID` |
| Get cloud machines of your own (the user confirms once), and wait until they are ready | `sheprd cloud create coder --count 3 --json`, `sheprd cloud wait BATCH --timeout 600 --json` |
| Run a command on a saved cloud machine (clone, build, commit, push) | `sheprd cloud exec ID --cwd ~/code/app -- git status` |
| Delete saved cloud machines you are done with, then follow it | `sheprd cloud destroy ID... --json`, `sheprd cloud wait BATCH --timeout 600 --json` |
| Check remote control and its daemon-host network interfaces, or pair a phone (Tailscale preferred; optionally choose an origin from status) | `sheprd remote status`, `sheprd remote pair [--address HTTPS_ORIGIN]` |
| Keep the machine awake during a long job | `sheprd caffeine on --for 2h` |
| Check whether the app runs, and which windows and hosts it has | `sheprd status` |

Add `--json` to any command for machine-readable output; errors then come
back as `{"error": {"code", "message"}}`. Exit statuses are the ones in the
table above. A command acts in the window that shows your workspace; it
exits 5 rather than act in another session's window.

Some commands only open the app's dialog and the user confirms there:
restoring a checkpoint, teleporting a worktree, fanning out a prompt, running
a worktree script the app has not seen, creating or destroying cloud
machines, and adding a cloud machine. Say so
when you run one, instead of waiting for it to finish. Run
`sheprd --help` to list every command, and `sheprd COMMAND --help`
for one.
