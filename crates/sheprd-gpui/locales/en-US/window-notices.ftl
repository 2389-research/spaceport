# The main window's brief notices: flashes over the terminal, the status
# line's errors, the config diagnostic cards, and the agent skill offer.

# A request to the daemon that could not be sent. $operation names it, as
# window-notices-navigate does or as the daemon's method name, and $error
# says why.
window-notices-request-failed = { $operation }: { $error }
# Moving focus to a workspace, tab, or pane.
window-notices-navigate = Navigate
window-notices-scrollback-needs-newer = Opening scrollback needs a newer Herdr daemon
window-notices-clear-needs-newer = Clear Pane needs a newer Herdr daemon.
window-notices-move-tabs-needs-newer = Moving tabs needs a newer Herdr daemon.

# The agent skill: installed where one agent looks ($path), or where
# several do ($count places).
window-notices-skill-installed-at = Installed the browser skill at { $path }
window-notices-skill-installed-places =
    { $count ->
        [one] Installed the browser skill in { $count } place
       *[other] Installed the browser skill in { $count } places
    }
# The one-time offer to install the agent skills.
window-notices-skill-offer-title = Let agents show you pages?
window-notices-skill-offer-body = Agents in your panes can open pages and HTML drafts in browser tabs here and read the notes you pin on them. The skill teaches them how.
window-notices-skill-offer-detail = Install writes spaceport/SKILL.md into ~/.claude/skills and ~/.agents/skills, whichever exist, and keeps it up to date. A skill of that name you wrote yourself is left alone. Remove it any time in Settings.
window-notices-skill-offer-install = Install
window-notices-skill-offer-decline = Not now

# Entering zen mode says how to leave it: by the command's name when it
# has no shortcut, else by its $key.
window-notices-zen-command = Zen mode: run Toggle Zen Mode to leave
window-notices-zen-key = Zen mode: { $key } to leave
# Shown once when a drag went to a program that takes the mouse.
window-notices-shift-select = Hold Shift and drag to select text

# The config diagnostic cards: the label of the app's own cards (the
# daemon's card is labeled with the device's name), the copied command
# chip, and the fold button's name for screen readers.
window-notices-diagnostic-app = Spaceport
window-notices-diagnostic-copied = Copied the command
window-notices-diagnostic-full = Full message
