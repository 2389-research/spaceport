# Asking before a pane or a tab closes.

# $agent is an agent's name, such as Claude.
close-modal-agent-waiting = { $agent } is waiting on you.
close-modal-agent-working = { $agent } is working.
close-modal-agents-busy = { $count } agents are working or waiting on you.
close-modal-pane-title = Close pane?
close-modal-tab-title = Close tab?
close-modal-pane-note = This terminates the pane and its running processes. This cannot be undone.
close-modal-tab-note = This terminates every pane and running process in this tab. This cannot be undone.
close-modal-do-not-ask = Do not ask again
close-modal-cancel = Cancel
close-modal-close-pane = Close pane
close-modal-close-tab = Close tab
close-modal-batch-title = Close { $count } tabs?
close-modal-batch-note = This terminates { $panes } Herdr terminal panes and their running processes, and closes { $views } client views. Core terminal processes keep running. Sent terminal closes cannot be undone.
close-modal-close-batch = Close { $count } tabs
close-modal-batch-stopped = Stopped after closing { $completed } of { $total } tabs. { $error }
close-modal-batch-pending = A sent terminal close may still finish. Review the remaining tabs before trying again.
close-modal-batch-timeout = The pending terminal close did not finish in time.
close-modal-batch-surface-timeout = The next terminal close could not pass the connection's surface barrier in time.
close-modal-batch-in-progress = A captured close batch is still in progress. Wait for its result before closing another batch.
close-modal-workspace-title = Close workspace?
close-modal-workspace-group-title = Close workspace group?
close-modal-workspace-note = This closes { $panes } terminal panes across { $workspaces } workspaces and can end their running processes. Checkout files stay on disk. This cannot be undone.
close-modal-close-workspace = Close workspace
close-modal-close-workspace-group = Close workspace group
close-modal-workspace-not-queued = The close request was not queued. Wait for pending commands or reconnect, then try again.
close-modal-workspace-unavailable = This action needs a connected Herdr TUI workspace.
