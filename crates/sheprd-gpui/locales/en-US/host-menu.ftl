# A saved device's menu, from its sidebar host header, and its pages:
# rename, remove, forward a port, and the terminal theme.

host-menu-rename = Rename…
# A switch: the device uses the keys its Herdr server binds.
host-menu-mute = Mute notifications
host-menu-server-keybindings = Use server keybindings
host-menu-forward-port = Forward port…
host-menu-theme = Terminal theme…
host-menu-remove = Remove device…
host-menu-local-keybindings = Using local keybindings: { $error }
host-menu-remove-failed = Remove { $device }: { $error }

# Removing a device. $device is its name, $target its SSH target,
# $session its Herdr session.
host-menu-remove-title = Remove device
host-menu-remove-ssh-note = Remove { $device } ({ $target } · { $session }) from this computer? Herdr keeps running on that host.
host-menu-cancel = Cancel
host-menu-remove-submit = Remove

host-menu-rename-title = Rename device
host-menu-close = Close
host-menu-rename-name = Name
host-menu-rename-submit = Rename
host-menu-renaming = Renaming…

# The device's forwarded ports, under its menu's rows, and the dialog that
# forwards one.
host-menu-forwards = Forwarded ports
host-menu-forward-connecting = connecting…
host-menu-forward-ended = ended
host-menu-forward-open = Open
host-menu-forward-dismiss = Dismiss
host-menu-forward-stop = Stop
host-menu-forward-title = Forward port
host-menu-forward-close = Close
host-menu-forward-intro = Reach a port that a program on { $device } listens on from this computer, over SSH. It stays forwarded until you stop it.
host-menu-forward-remote-port = Remote port
host-menu-forward-submit = Forward

# The device's terminal theme picker. $device is the device's name.
host-menu-theme-title = Terminal theme · { $device }
host-menu-theme-close = Close
host-menu-theme-search = Search themes...
# The row that drops the device's own theme for the app's.
host-menu-theme-same-as-app = Same as app
# Marks the theme the device uses now.
host-menu-theme-current = Current
host-menu-theme-hover-hint = Only this device's terminal changes. Hover to preview; Enter or click to save.
host-menu-theme-select-hint = Only this device's terminal changes; select the device to preview it.
