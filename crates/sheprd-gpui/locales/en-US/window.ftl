# The main window's own cards over the terminal area: the reconnecting card
# and the daemon's product announcement. Host names and the announcement's
# title and body come from the user and the daemon.

# The card over a dropped connection; $host is the device's name.
window-reconnecting = Reconnecting to { $host }
window-reconnect-now = Reconnect now
# The card in its place when this computer has no Herdr to start. Install
# Herdr opens the Herdr website; nothing is downloaded or run.
window-herdr-missing = Herdr isn't installed on this computer
window-herdr-missing-note = Your other machines still work, and Spaceport connects on its own once Herdr is installed.
window-install-herdr = Install Herdr
window-try-again = Try again
# The same card while the device is being removed on purpose.
window-removing = Removing { $host }
window-device-being-removed = It is being removed.
# The announcement's caption; $runtime owns the reported release version.
window-announcement = { $runtime } { $version }
window-announcement-preview = { $runtime } { $version } · preview
window-dismiss = Dismiss
# $error is why the daemon could not mark the announcement seen.
window-dismiss-announcement-failed = Dismiss announcement: { $error }
