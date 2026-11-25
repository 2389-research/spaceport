# The app's own failures, shown in a window, a dialog, or a notification,
# often inside another message's $error. $error is the failure underneath:
# a message of its own, or text from the system or a parser, kept as it is.
# $path is a file or folder.

# $operation is what failed on the note: create (its folder) or write.
error-agent-context = { $operation ->
    [create] Could not create the agent context note for the new checkout.
   *[write] Could not write the agent context note for the new checkout.
}
error-bundle-exists = { $path } already exists; choose a new file name
error-bundle-write = Could not write { $path }: { $error }
# $cleanup is why the half-written file could not be removed afterwards.
error-cleanup = { $error }; removing { $path }: { $cleanup }
error-connection-busy = Connection is busy. Try again.
error-connection-not-ready = The connection is not ready. Try again.
error-empty-tab-name = Enter a tab name.
error-empty-workspace-label = Workspace label must not be empty.
# $key is a config key, such as token.
error-image-animation-resize = Oversized GIF, WebP, and animated PNG images cannot be resized safely.
error-image-decode = Could not decode the oversized image.
error-image-decode-limit = The image exceeds the resize pixel or decoded-memory limit.
error-image-encode = Could not encode the resized image.
# $operation is what failed on the file: open, inspect, or read.
error-image-file = { $operation ->
    [open] Could not open the local image file.
    [inspect] Could not inspect the local image file.
   *[read] Could not read the local image file.
}
error-image-file-type = The local image must be a regular file.
error-image-format = SVG clipboard images are not supported. Use PNG, JPEG, GIF, WebP, BMP, or TIFF.
error-image-input-too-large = The image exceeds the { $limit }-byte resize input limit.
error-image-read-timeout = Reading the local image timed out (3 seconds).
error-image-size = The image must not be empty.
error-image-too-large = The image exceeds Herdr's { $limit }-byte upload limit.
error-invalid-external-url = Not an address with a scheme other than http, https, and file, of at most 8 KiB.
error-invalid-sidebar-width = invalid sidebar width
# sidebar_width_px is a config key.
error-invalid-stored-width = sidebar_width_px must be finite and positive, or null
error-invalid-window-state = Invalid saved window geometry or too many saved windows
error-invalid-worktree-notes = Invalid saved worktree notes
error-linked-file-missing = File not found: { $path }
# $host is the computer a terminal link names.
error-link-on-another-host = This link names a file on another computer, { $host }. It is not opened from here.
error-local-worktree-note = Worktree notes stay in this app and are never sent to the daemon
error-log-read = Could not read the log { $path }: { $error }
error-malformed-link = This link's address is not valid, so it opens nothing.
# HOME is an environment variable.
error-missing-home = HOME is not set
# XDG_STATE_HOME and HOME are environment variables.
error-missing-state-root = neither XDG_STATE_HOME nor HOME is set
error-no-snapshot = The daemon's current session data is not loaded; reconnect.
error-no-tab = No tab selected.
error-not-connected = Not connected to a daemon.
error-page-stack = The window has no view to place a page beneath
error-pane-image-decode = Could not decode an image a pane placed.
error-pane-image-limit = An image a pane placed exceeds the decoded size limit.
# A failure on a file or folder.
error-path = { $path }: { $error }
# XDG_CONFIG_HOME is an environment variable.
error-relative-config-root = XDG_CONFIG_HOME must be an absolute path
error-remote-agent-tools = Could not install the agent tools on the SSH host: { $error }
# $method is the daemon request that failed, such as workspace.create.
error-request = Could not send { $method } to the daemon: { $error }
error-screenshot-image = The page's picture could not be turned into a PNG
error-screenshot-write = Could not write the screenshot to { $path }: { $error }
error-selection-offscreen = The selection reaches rows the pane no longer shows.
error-selection-size = Selection is too large to copy.
error-selection-stale = The selected pane is no longer on screen.
error-settings-save = Could not finish saving Settings: { $error }
error-skill-install = Could not install the agent skill at { $path }: { $error }
error-stale-close-target = The original target changed or no longer exists. Cancel and try again.
error-stale-connection = The selected connection changed or is not ready. Cancel and try again.
error-stale-pane = The original pane changed or no longer exists. Cancel and try again.
error-stale-tab = The original tab changed or no longer exists. Cancel and try again.
error-stale-workspace = Workspace is no longer available. Dismiss and reopen the menu.
error-unsupported-command = This command action is not supported by this client.
