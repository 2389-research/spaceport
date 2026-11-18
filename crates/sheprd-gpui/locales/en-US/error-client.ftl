# Failures reaching a Herdr daemon on this computer, over SSH, or in WSL, and
# talking to it once connected. Most are shown in a connection notice or
# inside another message's $error, so they are written for the person reading
# them: what happened, then what to do, with the wire's own wording (a
# message, a pane screen, its session) kept only where it adds something.
# $error is the failure underneath, as the system or a parser words it.
# $status is how a process exited, in the system's words. `herdr update` is a
# command and stays as it is.

# $version is the Herdr version on the host, as the host reports it. $fix is
# who must update: app (this app) or daemon (Herdr on the host). $app is the
# client product name.
error-client-bridge-incompatible = { $fix ->
    [app] The Herdr daemon on this host cannot serve this app; update { $app }
   *[daemon] The Herdr daemon on this host cannot serve this app; run `herdr update` and reconnect
}
error-client-bridge-incompatible-version = { $fix ->
    [app] The Herdr daemon (version { $version }) on this host cannot serve this app; update { $app }
   *[daemon] The Herdr daemon (version { $version }) on this host cannot serve this app; run `herdr update` and reconnect
}
error-client-cancelled = The connection was stopped.
error-client-catalog-limit = The saved host list is larger than Spaceport stores; remove a host.
error-client-catalog-not-file = The saved host list is not a regular file; remove it so Spaceport can write a new one.
error-client-catalog-schema = The saved host list cannot be read; update Spaceport, or remove it to start over.
error-client-catalog-version-or-count = The saved host list is in a format this app does not know, or it lists too many hosts; update Spaceport or remove a host.
# $version is the oldest Herdr release this app works with.
error-client-closed-before-welcome = The host's Herdr daemon closed the connection without answering; it is likely older than { $version }, which this app requires. Run `herdr update` and reconnect
error-client-command-boot = The daemon's session changed before the command could run; reconnect and try again.
error-client-disconnected = The app is not connected to the daemon.
error-client-empty-surface = A pane screen was asked for with no rows or columns; nothing was sent.
error-client-encoded-surface-before-baseline = An encoded pane screen arrived before the full screen it belongs to; reconnect.
# The daemon refused a request without saying why.
error-client-endpoint = The daemon refused the request without saying why.
# $server_version is the daemon's version; $generation and $ours are the
# protocol versions the daemon and this app speak. $fix is who must update:
# app (this app) or daemon (Herdr).
error-client-endpoint-generation = { $fix ->
    [app] This app and the host's Herdr daemon (version { $server_version }) speak different protocol versions ({ $ours } here, { $generation } there); update { $app }
   *[daemon] This app and the host's Herdr daemon (version { $server_version }) speak different protocol versions ({ $ours } here, { $generation } there); run `herdr update` and reconnect
}
error-client-event-receiver-dropped = The connection's events stopped.
error-client-expected-welcome = The host did not greet this app as expected; run `herdr update` and reconnect.
error-client-forward-control = Could not prepare the port forward's private folder.
error-client-forward-exit = The SSH port forward ended ({ $status }); check host trust, authentication, and the network.
error-client-forward-local-port = No local port is free to forward to.
error-client-forward-refused = SSH could not listen on a local port ({ $status }).
error-client-forward-spawn = Could not start SSH for the port forward.
error-client-forward-timeout = SSH did not connect in time; check host trust and authentication.
error-client-frame-length = The daemon sent a message this app cannot read (bad message length); reconnect.
error-client-frame-prefix = The daemon sent a message this app cannot read (bad message prefix); reconnect.
error-client-full = Too many commands are waiting for the daemon; try again.
error-client-geometry-limit = The pane's screen is larger than the daemon allows; make the pane or window smaller.
error-client-handshake-timeout = The host did not finish connecting in time; check the network and reconnect.
error-client-health-timeout = The host did not answer its health check in time; check the network and reconnect.
error-client-incompatible-codecs = This app and the host's Herdr daemon cannot agree on a protocol; run `herdr update` and reconnect.
error-client-input-write-timeout = The daemon did not take the terminal input in time; reconnect.
error-client-invalid-session = That is not a valid session name.
error-client-io = Could not talk to the daemon: { $error }
error-client-json = The daemon's answer was not valid JSON: { $error }
error-client-legacy-daemon = This host's Herdr daemon is older than this app needs; run `herdr update` and reconnect.
error-client-missing-boot-id = That command needs the daemon's session first; reconnect and try again.
# $server_version is the version of Herdr on the SSH host.
error-client-missing-health-check = The Herdr daemon on this SSH host (version { $server_version }) has no health check this app can use; run `herdr update` there and reconnect
# $server_version is the daemon's version.
error-client-missing-surface-interest = The Herdr daemon on this host (version { $server_version }) cannot send pane screens the way this app asks; run `herdr update` and reconnect
error-client-no-local-socket = A host reached over SSH has no local socket to connect to.
error-client-partial-frame-timeout = The daemon started a message and never finished it; reconnect.
error-client-patch-before-baseline = A pane screen update arrived before the full screen it belongs to; reconnect.
error-client-profile-id = A saved host has an ID this app cannot use, or two hosts share one; remove one.
error-client-profile-label = A saved host's label cannot be used.
error-client-remote-command = The command on the host failed ({ $status }).
error-client-remote-output = The command on the host printed something this app did not expect.
error-client-request-timeout = The daemon did not answer in time; the request was not sent again.
error-client-response-boot = The daemon's session changed while the request was in flight; try again.
error-client-response-id = The daemon's answer did not match the request; reconnect.
error-client-response-limit = The daemon's answer was larger than this app accepts.
error-client-response-missing-result = The daemon's answer carried neither a result nor a reason.
error-client-response-schema = The daemon's answer is not one this app can read.
error-client-response-type = The daemon answered a different request than the one sent.
error-client-script-cancelled = host script cancelled
error-client-script-connect = could not reach host for script: { $error }
# $stderr is the end of what the script printed on the host, as it printed it.
error-client-script-exit = host script failed ({ $status }): { $stderr }
error-client-script-input = could not read host script input
error-client-script-io = host script I/O failed
error-client-script-output = could not write host script output
error-client-script-output-limit = host script output exceeds limit
error-client-script-spawn = could not start host script
error-client-script-timeout = host script made no progress before its deadline
error-client-script-unsupported = Host scripts need this app running on Linux or macOS.
error-client-script-worker = host script worker panicked
error-client-selection-limit = The saved host selection is larger than Spaceport stores; remove it to start over.
error-client-selection-not-file = The saved host selection is not a regular file; remove it so Spaceport can write a new one.
error-client-selection-schema = The saved host selection cannot be read; remove it so Spaceport can write a new one.
error-client-selection-unavailable = The selected host is not there, or it is turned off; pick another host.
error-client-selection-version = The saved host selection is in a format this app does not know; update Spaceport.
error-client-snapshot-identity = The daemon's session changed, or its screen data went backwards; reconnect.
error-client-socket-closed = The daemon closed the connection.
error-client-storage-destination-not-file = Spaceport's saved data is not a regular file.
error-client-storage-path = Spaceport's saved data has a path it cannot use.
error-client-surface-before-snapshot = A pane screen arrived before the daemon's session data; reconnect.
error-client-surface-encoding-not-negotiated = This Herdr daemon did not offer the screen encoding this app asked for; run `herdr update` and reconnect.
error-client-surface-identity = A pane screen arrived for the wrong pane or an out-of-date revision; reconnect.
error-client-unsolicited-response = The daemon answered a request this app never sent; reconnect.
error-client-unsupported-method = This Herdr daemon does not offer that request; run `herdr update` and reconnect.
error-client-unsupported-surface-interest = This Herdr daemon cannot send pane screens the way this app asks; run `herdr update` and reconnect.
error-client-upload-cancelled = SSH file transfer cancelled
# $error is why the upload failed; the temporary files it left on the host
# could not be removed either.
error-client-upload-cleanup = { $error }; remote temporary-file cleanup also failed
error-client-upload-cleanup-path = Cleaning up the files left on the host needs the paths the upload returned.
error-client-upload-exit = SSH file transfer failed; check host trust, authentication, and remote storage
error-client-upload-io = SSH file transfer I/O failed
error-client-upload-name = file transfer requires a UTF-8 basename without controls
error-client-upload-not-file = file transfer source is not a regular file
error-client-upload-path-limit = SSH file transfer accepts at most 256 paths
error-client-upload-response = invalid or excessive SSH file transfer response
error-client-upload-size-overflow = The files are too large to transfer at once.
error-client-upload-source-changed = file transfer source changed length
error-client-upload-timeout = SSH file transfer made no progress for 30 seconds
error-client-upload-unsupported = SSH file transfer needs this app running on Linux or macOS.
error-client-welcome-kind = The host's Herdr daemon sent a greeting this app cannot read; run `herdr update` and reconnect.
# $code and $message are the daemon's reason for refusing the connection,
# in its own words.
error-client-welcome-rejected = The host's Herdr daemon refused the connection ({ $code }): { $message }
