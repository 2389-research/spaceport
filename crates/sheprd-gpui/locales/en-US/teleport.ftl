# Shared labels for pane moves and host dispatch. Names and commands are data.

# What a pane runs on the destination.
teleport-pane-shell = Shell
teleport-pane-resume = { $agent }: resume session
teleport-pane-handoff-again = { $program }: handoff note, then start again
teleport-pane-handoff-to = { $program }: handoff note, continued by { $agent }
teleport-pane-start = Start { $command }
teleport-pane-run = Run { $command }
teleport-pane-missing = { $program }: not installed there, skipped

# The step a move is on.
teleport-step-starting = Starting
teleport-step-discover = finding matching repositories
teleport-step-review = reviewing the move
teleport-step-clone = copying the repository to the destination
teleport-step-open = opening the repository on the destination
teleport-step-handoff = asking agents for handoff notes
teleport-step-capture = capturing commits and changes
teleport-step-transfer = copying changes to the destination
teleport-step-fetch = fetching the branch on the destination
teleport-step-create-worktree = creating the destination worktree
teleport-step-restore = restoring uncommitted changes
teleport-step-tabs = recreating tabs
teleport-step-retire = stopping programs in the source workspace
teleport-step-credentials = installing GitHub credentials on the destination
teleport-step-sessions = moving agent sessions
teleport-step-launch = starting programs

# What did not carry over in a move that otherwise finished.
teleport-warning-started-afresh = { $agent } started afresh: { $error }
teleport-warning-envrc = The repository tracks .envrc, so GH_TOKEN was not added; git pull and push still work
teleport-warning-github = GitHub access was not set up: { $error }
teleport-warning-no-note-from = No handoff note from { $pane }: { $error }
teleport-warning-no-note = No handoff note: { $error }
teleport-warning-missing = { $program } is not installed on { $host }
teleport-warning-pane-open = The source pane stayed open: { $error }
