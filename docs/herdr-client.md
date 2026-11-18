# Using Herdr with Spaceport

Herdr owns its sessions and terminal processes.
Closing Spaceport leaves them running.
After an explicit Herdr shutdown, Spaceport waits for **Start Herdr** or **Terminal → Reconnect**.
Ordinary connection loss still uses automatic reconnect.

Herdr TUI (also called Herdr 1) must be version **0.9.0 or later**.
Spaceport provides native windows, rich chat views, and a desktop control CLI around shared Herdr terminal sessions.
The terminal client and Spaceport share daemon state, including focused panes and release-note dismissal.
Spaceport keeps its appearance and keyboard settings in its own files.
It does not import Herdr's terminal-client configuration.
Use Herdr's CLI for runtime operations Spaceport does not expose.
Spaceport excludes the inherited IDE, Git, cloud-provisioning, and embedded-browser controls.

Open a workspace's menu and choose **Close workspace** to end that workspace.
The confirmation lists every affected workspace and the number of terminal panes.
Closing a primary workspace can close its linked workspace group, as in Herdr TUI.
Closing a linked workspace closes only that workspace.
Checkout files stay on disk.
The command palette and `spaceport run close_workspace` open the same confirmation.
The CLI only opens the dialog; it does not confirm termination.
The default Herdr prefix chord is `Ctrl-B`, then `Shift-D`.

## Optional Herdr Core experiment

Herdr TUI remains the default runtime.
Herdr Core is a separate experiment under **Settings → General → Experimental → Enable Herdr Core**.
Save the setting, then restart Spaceport to apply it.
Enabling Core exposes its controls; it does not change the runtime of existing tabs.

With Core enabled, **Settings → Tabs → Use Herdr Core for new local tabs** changes **New Tab** on this computer.
Those terminals belong to Core and do not appear in Herdr TUI or its `herdr` CLI.
Tabs on SSH hosts still use their host's Herdr TUI.
Tabs inside an existing Core group always use that group's Core.

Turning Core off and restarting hides Core tabs without ending their terminals.
To show the tabs again, enable Core and restart Spaceport.
Core groups organize terminal references in Core's shared document.
Window split groups arrange views inside one Spaceport window.
Splitting a window does not create another Core group.

## Names and shortcut settings

**Herdr TUI** is the runtime also called Herdr 1.
**Herdr Core** is the separate runtime previously labeled Herdr 2.
Executable names such as `herdr-core`, and command examples using `hc`, keep their actual names.
A **device** is a saved Spaceport connection to a computer.
The CLI's `host` family addresses these devices.
A **Core machine** is a route reported by Core's own machine catalog.
Those catalogs have separate owners.

Spaceport's `[keybindings]` table uses the names from `spaceport commands`.
Its private `[keys]` table uses Herdr-compatible prefix action names plus documented Spaceport actions.
For example, `[keybindings].split_right` corresponds to `[keys].split_vertical`.
Likewise, `split_down` corresponds to `split_horizontal`, and `toggle_zoom` corresponds to `zoom`.
These file keys keep their established names; menu labels use **Split Right** and **Split Down**.

## SSH connections

Spaceport uses noninteractive SSH with `BatchMode=yes` and no password prompts.
It checks host keys strictly unless the target explicitly skips verification.
For an unknown or changed key, verify the host in a terminal before reconnecting.
Spaceport does not automatically accept a key.

For password or MFA authentication, configure your own `ControlMaster` and `ControlPath` in SSH config.
Authenticate with `ssh HOST` in a terminal first.
Spaceport uses `ControlMaster=no` for its terminal connection and can reuse that existing master.
It cannot prompt for authentication when the master expires.

Verified hosts follow your SSH agent-forwarding setting.
Targets that skip host-key checks disable agent and X11 forwarding, even when SSH config enables them.

The desktop control socket stays on the computer running Spaceport.
An agent on an SSH host cannot reach it through the terminal connection.
See the [CLI guide](cli.md) for local commands and targeting.

## Herdr release notes

Closing Herdr's release-notes sheet marks that version read on the owning daemon.
The change applies to every client, including Herdr's TUI.
Spaceport's application updates use their separate update route.
