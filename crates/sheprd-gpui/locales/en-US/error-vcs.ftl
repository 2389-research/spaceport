# Git on the checkouts the app shows: read-only probes and close checks.
# Git's own output stays as it is in $details.

## Read-only Git checks.

# $kind is the step that failed. $operation is its English description,
# shown for a step not listed here. $details is Git's own output.
error-vcs-git-failed = { $kind ->
    [check-uncommitted] git check uncommitted files failed: { $details }
    [check-unpushed] git check unpushed commits failed: { $details }
    [count-working-tree] git count working tree changes failed: { $details }
    [fetch-pull-request] git fetch the pull request branch failed: { $details }
    [find-branch-start] git find where the branch started failed: { $details }
    [find-checkout] git find the checkout failed: { $details }
    [list-attribute-files] git list attribute files failed: { $details }
    [list-branches] git list branches failed: { $details }
    [list-untracked] git list untracked files failed: { $details }
    [list-working-tree] git list working tree changes failed: { $details }
    [prepare-remote-checkout] git prepare the remote pull request or issue checkout failed: { $details }
    [read-attributes] git read attributes failed: { $details }
    [read-default-branch] git read the remote's default branch failed: { $details }
    [read-head] git read HEAD failed: { $details }
    [read-working-tree-changes] git read working tree changes failed: { $details }
    [read-working-tree-state] git read working tree state failed: { $details }
   *[other] git { $operation } failed: { $details }
}
# $kind is the step that failed. $operation is its English description,
# shown for a step not listed here.
error-vcs-git-process = { $kind ->
    [start-worker] Could not start the Git worker.
   *[other] Could not { $operation }.
}
error-vcs-git-worker = Git worker stopped. Retry the operation.
