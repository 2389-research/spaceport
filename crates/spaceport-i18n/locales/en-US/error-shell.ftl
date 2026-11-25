# Why a shell on a host could not start or finish a step, shown beneath the
# failure of the feature that ran it, such as a listening-ports scan.

# $kind is the step that failed. $operation is its English description,
# shown for a step not listed here.
error-shell-process = { $kind ->
    [host-shell] Could not start the host shell.
   *[other] Could not { $operation }.
}
error-shell-size = Host shell output exceeded the size limit.
error-shell-timeout = Host shell step timed out.
error-shell-unreachable = Could not reach a shell on this host.
error-shell-wsl-host-unsupported = WSL distributions can only be reached from Windows.
