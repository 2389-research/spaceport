# The Add Device dialog: its frame and rail, and the SSH host form.

add-device-title = Add Device
add-device-close = Close
# The rail's entries. WSL keeps its own name.
add-device-source-host = SSH host
# The device picker's line under "Add Device…": what this build can add.
add-device-offer-ssh-wsl = An SSH host or a WSL distribution
add-device-offer-ssh = Set up a remote host over SSH

# The SSH form. $target is what the user typed, such as user@host or an
# SSH alias.
add-device-target-placeholder = user@hostname or SSH alias
add-device-label-placeholder = The SSH target
add-device-core-intro = Add a machine that already runs Herdr Core. Core checks the SSH target without installing software or changing host-key trust.
add-device-core-target-required = Enter an SSH target or alias.
add-device-core-connecting = Connecting to the local Core before adding { $target }…
add-device-core-adding = Core is checking and adding { $target }…
add-device-core-waiting = Another window is adding { $target }. Wait for its result.
add-device-core-unavailable = Local Core is unavailable: { $reason }
add-device-core-disconnected = The local Core connection ended. Check Core and submit again.
add-device-core-worker-failed = Could not start the Core add request: { $reason }
add-device-core-unknown = Core did not confirm whether { $target } was added. Check Core's machine list. Spaceport will not send another add request this run. Restarting Spaceport does not prove the outcome.
add-device-target = SSH target
add-device-label = Label (optional)
add-device-session = Remote session (optional)
add-device-submit = Add device
add-device-submit-checking = Checking…
