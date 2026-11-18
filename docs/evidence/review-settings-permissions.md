# Settings file permissions

Date: 2026-11-17. Scope: `tbg4` under `9s44`.

## Acceptance and change

Replacing existing settings must retain the target file's permissions.
Saving through a dotfile symlink must retain that symlink and its target permissions.
New files retain the private temporary-file default.
Unix saves sync the parent directory after publishing the file.
Windows directory syncing remains unavailable through the existing standard-library implementation.

The writer reads the existing metadata before creating its replacement.
It restores Unix owner and group before applying permissions.
It syncs that metadata before the atomic rename.
Metadata errors propagate instead of silently applying different permissions.
Both create and replace sync their parent directories on Unix.

## Verification

The real-file regression failed before the change: mode 0640 became 0600.
Independent review caught a group-ownership change despite matching mode bits.
The supplementary-group regression failed before ownership restoration.
Eight settings-file tests passed after the complete fix.
They cover direct files, symlink targets, private files, and refusal to replace an existing creation target.
The executable CLI regression passed against private file and symlink fixtures, including owner and group checks.
Independent data-integrity review approved the final ownership, permissions, and publish ordering.
On systems without an alternate supplementary group, the test reports that limitation and checks the current identity.
The alternate-group boundary ran on this Mac.

Logs: `/tmp/spaceport-9s44-settings-permissions-{red,green,executable}.log`.
Final logs: `/tmp/spaceport-9s44-settings-ownership-{red,green}.log` and `settings-permissions-executable-final.log`.
