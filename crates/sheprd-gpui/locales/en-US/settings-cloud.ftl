# Settings > Cloud devices: what the page and every provider's card say
# around their own fields. Provider, machine, region, size, and image names
# come from the providers and stay as they are. The section and the New
# device setup card take their titles from settings.ftl.

settings-cloud-unavailable = Cloud devices are unavailable on Windows. Accounts can still be edited here for other systems.
settings-cloud-remove = Remove
settings-cloud-none-yet = None yet.

## Shared by the provider cards.

settings-cloud-save = Save
# A job the main window runs; $name is the new device's name.
settings-cloud-setup-note = An explicit setup rerun on an existing cloud device uses the agents switched on here, then the setup script from [cloud] in the config file. A provider's table, such as [cloud.container], replaces either.
# $agent is the agent's product name, such as Claude Code.
settings-cloud-setup-install = Install { $agent }
settings-cloud-setup-none = No setup script of your own.
# Under the first lines of the user's script, how many more it has.
settings-cloud-setup-more = { $count ->
        [one] … { $count } more line
       *[other] … { $count } more lines
    }
settings-cloud-setup-edit = Edit setup script in config file

## Images: each project's sandbox image at a provider.
