# Failures reaching a Herdr daemon (the "Herdr server") on this computer,
# over SSH, or in WSL, and talking to it once connected. Most are shown in
# a connection notice or inside another message's $error. An "endpoint" is
# the daemon's connection protocol; a "surface" is a pane's screen contents;
# a "boot" is one run of the daemon. $error is the failure underneath, as
# the system or a parser words it. $status is how a process exited, in the
# system's words. `herdr update` is a command and stays as it is.

# $version is the Herdr version on the host, as the host reports it. $fix is
# who must update: app (this app) or daemon (Herdr on the host). $app is the
# client product name.
error-client-bridge-incompatible = { $fix ->
    [app] This device's Herdr version needs a newer app. Update { $app }.
   *[daemon] This device's Herdr version needs an update. Run `herdr update`, then choose Terminal → Reconnect.
}
error-client-bridge-incompatible-version = { $fix ->
    [app] Herdr { $version } on this device needs a newer app. Update { $app }.
   *[daemon] Herdr { $version } on this device needs an update. Run `herdr update`, then choose Terminal → Reconnect.
}
error-client-cancelled = The connection stopped.
error-client-catalog-limit = The saved connection list is too large.
error-client-catalog-not-file = The saved connections path is not a file.
error-client-catalog-schema = The saved connections file could not be read.
error-client-catalog-version-or-count = The saved connections use an unsupported format or contain too many entries.
# $version is the oldest Herdr release this app works with.
error-client-closed-before-welcome = Herdr closed the connection without answering. It may be older than the required version { $version }. Run `herdr update`, then choose Terminal → Reconnect.
error-client-command-boot = This command does not match the current connection. Choose Terminal → Reconnect.
error-client-disconnected = Spaceport is disconnected from Herdr.
error-client-empty-surface = The terminal view needs a width and height. Resize the window and try again.
error-client-encoded-surface-before-baseline = Herdr sent a screen update before the initial screen. Choose Terminal → Reconnect.
# The server refused a request without saying why.
error-client-endpoint = Spaceport could not read Herdr's error explanation.
# $server_version is the daemon's version; $generation and $ours are the
# protocol generations the daemon and this app speak. $fix is who must
# update: app (this app) or daemon (Herdr).
error-client-endpoint-generation = { $fix ->
    [app] Herdr { $server_version } needs a newer app. Update { $app }. Connection formats: Herdr { $generation }, app { $ours }.
   *[daemon] Herdr { $server_version } needs an update. Run `herdr update`, then choose Terminal → Reconnect. Connection formats: Herdr { $generation }, app { $ours }.
}
error-client-event-receiver-dropped = Spaceport stopped receiving connection updates.
error-client-expected-welcome = Herdr did not send the expected connection information. Choose Terminal → Reconnect.
error-client-forward-control = could not create the port forward's private control directory
error-client-forward-exit = SSH port forward ended ({ $status }); check host trust, authentication, and the network
error-client-forward-local-port = no local port is free to forward to
error-client-forward-refused = SSH could not listen on a local port ({ $status })
error-client-forward-spawn = could not start SSH for the port forward
error-client-forward-timeout = SSH did not connect in time; check host trust and authentication
error-client-frame-length = Herdr sent terminal data Spaceport could not read. Choose Terminal → Reconnect.
error-client-frame-prefix = Herdr sent terminal data Spaceport could not read. Choose Terminal → Reconnect.
error-client-full = Too many commands are waiting. Let them finish before trying again.
error-client-geometry-limit = This terminal size is larger than Herdr supports. Resize the window and try again.
error-client-handshake-timeout = Herdr did not finish opening the connection in time. Choose Terminal → Reconnect.
error-client-health-timeout = Herdr stopped answering connection checks. Choose Terminal → Reconnect.
error-client-incompatible-codecs = This version of Herdr uses a terminal format Spaceport does not support.
error-client-input-write-timeout = Spaceport could not finish sending terminal input. Some input may have arrived. Choose Terminal → Reconnect.
error-client-invalid-session = invalid session name
error-client-io = Could not communicate with Herdr: { $error }
error-client-json = Spaceport could not read Herdr's reply: { $error }
error-client-legacy-daemon = This Herdr version is too old for Spaceport. Run `herdr update`, then choose Terminal → Reconnect.
error-client-missing-boot-id = The connection is not ready for this command. Wait for it to connect.
# $server_version is the version of Herdr on the SSH host.
error-client-missing-health-check = Herdr { $server_version } on this device cannot report connection health. Run `herdr update` there, then choose Terminal → Reconnect.
# $server_version is the daemon's version.
error-client-missing-surface-interest = Herdr { $server_version } cannot safely share these terminal views. Run `herdr update`, then choose Terminal → Reconnect.
error-client-no-local-socket = This remote connection has no local socket file.
error-client-partial-frame-timeout = Herdr stopped partway through a terminal update. Choose Terminal → Reconnect.
error-client-patch-before-baseline = Herdr sent a screen update before the initial screen. Choose Terminal → Reconnect.
error-client-profile-id = A saved connection has a missing, invalid, or repeated ID.
error-client-profile-label = A saved connection has an invalid name.
error-client-remote-command = remote command failed ({ $status })
error-client-remote-output = remote command returned unexpected output
error-client-request-timeout = Herdr did not confirm the request. It may have happened; check the result before trying again.
error-client-response-boot = Herdr's reply belongs to another connection. Choose Terminal → Reconnect.
error-client-response-id = Herdr's reply did not match the waiting request. Choose Terminal → Reconnect.
error-client-response-limit = Herdr's reply was too large for Spaceport to read.
error-client-response-missing-result = Herdr's reply contained neither a result nor an explanation.
error-client-response-schema = Spaceport could not read Herdr's reply.
error-client-response-type = Herdr replied to a different action. Choose Terminal → Reconnect.
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
error-client-script-unsupported = host scripts require a Linux or macOS client
error-client-script-worker = host script worker panicked
error-client-selection-limit = The saved connection choice is too large.
error-client-selection-not-file = The saved connection choice path is not a file.
error-client-selection-schema = The saved connection choice could not be read.
error-client-selection-unavailable = The selected connection is missing or disabled. Select another device.
error-client-selection-version = The saved connection choice uses an unsupported format.
error-client-snapshot-identity = Herdr restarted or sent older session state. Choose Terminal → Reconnect.
error-client-socket-closed = The connection to Herdr closed.
error-client-storage-destination-not-file = storage destination is not a regular file
error-client-storage-path = invalid storage path
error-client-surface-before-snapshot = Herdr sent a screen before its session information. Choose Terminal → Reconnect.
error-client-surface-encoding-not-negotiated = Herdr sent a terminal format this connection did not agree to use. Choose Terminal → Reconnect.
error-client-surface-identity = Herdr's terminal screen does not match its session information. Choose Terminal → Reconnect.
error-client-unsolicited-response = Herdr sent a reply without a matching request. Choose Terminal → Reconnect.
error-client-unsupported-method = This version of Herdr does not support this action.
error-client-unsupported-surface-interest = This version of Herdr cannot safely share these terminal views.
error-client-upload-cancelled = SSH file transfer cancelled
# $error is why the upload failed; the temporary files it left on the host
# could not be removed either.
error-client-upload-cleanup = { $error }; remote temporary-file cleanup also failed
error-client-upload-cleanup-path = cleanup requires unchanged absolute paths returned by upload_files
error-client-upload-exit = SSH file transfer failed; check host trust, authentication, and remote storage
error-client-upload-io = SSH file transfer I/O failed
error-client-upload-name = file transfer requires a UTF-8 basename without controls
error-client-upload-not-file = file transfer source is not a regular file
error-client-upload-path-limit = SSH file transfer accepts at most 256 paths
error-client-upload-response = invalid or excessive SSH file transfer response
error-client-upload-size-overflow = file transfer total size exceeds u64
error-client-upload-source-changed = file transfer source changed length
error-client-upload-timeout = SSH file transfer made no progress for 30 seconds
error-client-upload-unsupported = SSH file transfer requires a Linux or macOS client
error-client-welcome-kind = Herdr sent unsupported connection information. Choose Terminal → Reconnect.
# $code and $message are the daemon's reason for refusing the connection,
# in its own words.
error-client-welcome-rejected = { $code }: { $message }
