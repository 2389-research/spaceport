# Why removing or renaming a saved device failed.
# $status is how Herdr's command exited, in the system's words; $detail is
# what it printed.

error-devices-busy = This device is already being removed. Wait for that to finish, then try again.
error-devices-remove = Removing the device failed ({ $status })
error-devices-remove-detail = Removing the device failed ({ $status }): { $detail }
error-devices-rename = Renaming the device failed ({ $status })
error-devices-rename-detail = Renaming the device failed ({ $status }): { $detail }
error-devices-setup-no-profile = This device has no saved profile.
error-devices-setup-label = Device names are at most 128 bytes, without control characters.
error-devices-setup-timeout = Saving the device timed out
