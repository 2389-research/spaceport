# Starting or talking to the Herdr daemon. Terminal > Reconnect is a menu
# command.

error-daemon-cancelled = Daemon startup cancelled.
# $status is how the daemon ended, in the system's words.
error-daemon-exited = The Herdr daemon exited before accepting connections: { $status }. Check the daemon's log.
# $error is the system's text.
error-daemon-spawn = Could not start the Herdr daemon: { $error }. Use Terminal > Reconnect to retry.
# $path is the daemon's socket.
error-daemon-timeout = Timed out waiting for the Herdr daemon at { $path }. Check the daemon's log and use Terminal > Reconnect.
