# A window's devices: connecting to them, switching between them, and saving
# which one it shows. Errors appear in the main window's status line.

# The reason shown while reconnecting when the connection gave none.
endpoint-connection-lost = connection lost
# A device's state in the device picker. "Herdr update needed" means the
# device's Herdr is too old for this app; "app update needed" means this app
# is too old for it.
endpoint-status =
    { $state ->
        [disabled] disabled
        [detached] detached
        [daemon-outdated] Herdr update needed
        [client-outdated] app update needed
        [herdr-missing] Herdr not installed
        [herdr-stopped] Herdr stopped
        [reconnecting] reconnecting
        [online] online
       *[connecting] connecting
    }
# A pane program copied text to the clipboard with OSC 52. It is worded apart
# from the user's own selection copy (window-find-copied) because the text came
# from inside the terminal, not from a gesture of theirs.
endpoint-copied = copied from the terminal
# $error says why.
endpoint-save-selection-failed = Save host selection: { $error }
endpoint-catalog-failed = Host catalog: { $error }
endpoint-reload-config-failed = Reload config: { $error }
endpoint-activate-failed = Activate: { $error }
# $device is the device's name.
endpoint-activation-failed = { $device }: surface activation failed or timed out
# $reason is what the connection reported.
endpoint-activation-failed-because = { $device }: surface activation failed or timed out: { $reason }
endpoint-surface-switching-unsupported = Current endpoint does not support surface switching
