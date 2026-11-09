# A repository's worktree scripts (.herdr/worktree.toml): setup when a
# worktree is created and run from the workspace menu. $kind is setup or run;
# $path is the file.

worktree-scripts-none =
    { $kind ->
        [setup] No setup script in { $path }
       *[run] No run script in { $path }
    }
# $error says why.
worktree-scripts-not-started =
    { $kind ->
        [setup] The setup script did not start: { $error }
       *[run] The run script did not start: { $error }
    }

# The page that asks before a script runs.
worktree-scripts-ask-setup = Run this worktree's setup script?
worktree-scripts-ask-run = Run this worktree's run script?
# $repo is the repository's name; $tab is the new tab's name, as the
# script's key in the file names it.
worktree-scripts-asks = { $repo } asks to run this from { $path }, in a new “{ $tab }” tab of its checkout:
worktree-scripts-others = Trusting the file also trusts its other scripts:
# $host is the device the daemon runs on.
worktree-scripts-trust-note = Scripts run with sh on { $host }, the daemon's host. Trust is remembered for { $repo } there until the file changes.
worktree-scripts-trust-run = Trust and run
worktree-scripts-skip = Don't run
