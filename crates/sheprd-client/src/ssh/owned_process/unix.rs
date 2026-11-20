// ABOUTME: Shares the pipe watchdog used by owned SSH masters and tunnels.
// ABOUTME: Ends only its direct child and removes its private control directory.
use std::{
    io,
    path::Path,
    process::{Command, Stdio},
};

/// Runs the master (`"$@"`) until its owner closes the stdin pipe.
/// Copy the pipe to fd 3 because background jobs receive `/dev/null` on stdin.
/// Send SIGTERM, then SIGKILL because GUI children may inherit ignored SIGTERM.
/// Kill and reap the watcher when the master exits, before its PID can be reused.
/// Remove only the caller's private control socket and directory.
pub(crate) const WATCHDOG: &str = r#"dir=$1
shift
exec 3<&0
"$@" </dev/null 3<&- &
master=$!
(
    read -r _ <&3
    kill "$master" 2>/dev/null
    sleep 1
    kill -9 "$master" 2>/dev/null
) &
watcher=$!
exec 3<&-
wait "$master"
status=$?
kill -9 "$watcher" 2>/dev/null
wait "$watcher" 2>/dev/null
if [ -n "$dir" ]; then
    rm -f "$dir/c" "$dir"/c.*
    rmdir "$dir" 2>/dev/null
fi
exit "$status""#;

/// Wraps the executable, arguments, explicit environment, and working directory.
/// The caller sets output streams. Stdin is the ownership pipe.
pub(crate) fn command(child: &Command, directory: Option<&Path>) -> io::Result<Command> {
    let path = child
        .get_envs()
        .find(|(key, _)| *key == "PATH")
        .map(|(_, value)| value.map(ToOwned::to_owned))
        .unwrap_or_else(|| std::env::var_os("PATH"));
    let cwd = std::env::current_dir()?;
    let cwd = child
        .get_current_dir()
        .map_or(cwd.clone(), |directory| cwd.join(directory));
    let executable = which::which_in(child.get_program(), path, cwd)
        .map_err(|error| io::Error::new(io::ErrorKind::NotFound, error))?;
    let mut command = crate::new_command("/bin/sh");
    command
        .args(["-c", WATCHDOG, "spaceport-ssh"])
        .arg(directory.unwrap_or_else(|| Path::new("")))
        .arg(executable)
        .args(child.get_args())
        .stdin(Stdio::piped());
    for (key, value) in child.get_envs() {
        match value {
            Some(value) => {
                command.env(key, value);
            }
            None => {
                command.env_remove(key);
            }
        }
    }
    if let Some(directory) = child.get_current_dir() {
        command.current_dir(directory);
    }
    Ok(command)
}
