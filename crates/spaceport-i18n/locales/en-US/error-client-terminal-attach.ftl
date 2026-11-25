# Failures attaching directly to one Herdr terminal.

error-client-terminal-attach-busy = too many terminal scrolls are waiting
error-client-terminal-attach-ended = the terminal attach has ended
error-client-terminal-attach-io = terminal attach I/O failed
error-client-terminal-attach-protocol = The daemon sent a message this app cannot read while attaching the terminal.
error-client-terminal-attach-refused = Herdr refused the terminal attach
# $reason is Herdr's own message, kept as it is.
error-client-terminal-attach-refused-reason = Herdr refused the terminal attach: { $reason }
# $version is a protocol version number.
error-client-terminal-attach-rejected = Herdr refused the terminal attach (protocol { $version })
error-client-terminal-attach-scroll = a terminal scroll must move at least one line
error-client-terminal-attach-size = terminal size must be nonzero
error-client-terminal-attach-unexpected = Herdr answered the terminal attach with a message this app did not expect.
