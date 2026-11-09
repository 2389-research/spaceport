# Why an AI service's plan usage could not be read, shown in the status bar
# and the usage panel. None of it carries account details.

error-usage-connect = Could not reach the usage service from this host.
error-usage-no-plan = This account has no plan with usage limits to show.
# $kind is the step that failed. $operation is its English description,
# shown for a step not listed here.
error-usage-process = { $kind ->
    [remote-shell] Could not start the remote usage shell.
   *[other] Could not { $operation }.
}
error-usage-rate-limited = Usage is rate limited.
error-usage-size = Usage output exceeded the size limit.
# $status is the HTTP status code.
error-usage-status = Usage service returned HTTP { $status }.
error-usage-timeout = Usage check timed out.
error-usage-unreachable = Could not reach this host over SSH to read usage.
error-usage-wsl-host-unsupported = WSL distributions can only be reached from Windows.
