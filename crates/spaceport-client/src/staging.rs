//! The shell that stages one private temporary file on a POSIX host, and the
//! shape of the path that shell reports back.
//!
//! The desktop upload and the phone's screenshot both run these scripts over
//! their own transport, so the rules a host is held to live here: a private
//! umask, a `herdr-upload.` directory under the host's own `${TMPDIR:-/tmp}`
//! resolved through symlinks (macOS links `/tmp` to `/private/tmp`, so the
//! path a host prints is not one a client could guess), an exclusive create,
//! a byte count that must match what was sent, and a trap that removes the
//! directory again unless the client kept the file.
//!
//! A script names its destination in `$file` and opens it on descriptor 3, so
//! the exit trap can only remove a file the host created. Each directive below
//! is one line in both directions, and each script prints only what the client
//! parses: nothing here is a general shell library.

use crate::script::shell_quote;

/// The longest staging path or directive line a client accepts.
pub const LINE_LIMIT: usize = 4096;

/// The staging directory's name before `mktemp` fills in twelve characters.
const DIRECTORY: &str = "herdr-upload.XXXXXXXXXXXX";

/// What every staging directory's name starts with.
const PREFIX: &str = "herdr-upload.";

/// How many random characters `mktemp` fills in after [`PREFIX`].
const SUFFIX: usize = 12;

/// The host announced an empty staging directory; the next line is its path.
pub const READY: &str = "herdr-upload-ready:1";

/// The host created the file; the next line is its path.
pub const CREATED: &str = "herdr-upload-created:1";

/// The host holds the sent bytes; the next line is its path.
pub const COMPLETE: &str = "herdr-upload-complete:1";

/// The client accepts the announced directory and will stream bytes.
pub const ACCEPT: &str = "accept";

/// The client keeps the staged file and will name that path in its terminal.
pub const RETAIN: &str = "retain";

/// The host holds the retained file; the connection may close from here.
pub const RETAINED: &str = "retained";

/// The client will not use the file; the host removes it and its directory.
pub const DISCARD: &str = "discard";

/// The whole script for a client that streams `len` bytes after the host
/// announces a directory. The client answers [`ACCEPT`], reads [`CREATED`]
/// and the path before it streams, and reads [`COMPLETE`] and the path once
/// the host has counted the bytes. The file then persists until the client
/// asks separately for it to be removed.
pub fn receive_script(name: &str, len: u64) -> String {
    format!(
        "{}\
        printf '{READY}\\n%s\\n' \"$dir\" || exit 1\n\
        IFS= read -r accept || exit 1\n\
        [ \"$accept\" = {ACCEPT} ] || exit 1\n\
        {}\
        printf '{CREATED}\\n%s\\n' \"$file\" || exit 1\n\
        cat >&3 || exit 1\n\
        exec 3>&-\n\
        {}\
        printf '{COMPLETE}\\n%s\\n' \"$file\" || exit 1\n\
        complete=1\n",
        staging_directory(),
        create_exclusively(name),
        verify_size(len),
    )
}

/// The whole script for a client that decides afterwards whether to keep the
/// file. It reads exactly `len` bytes, so the reads that follow cannot race
/// the transfer's noise, and prints [`RETAINED`] only once the file is the
/// client's to name. A following [`DISCARD`] line removes it again; closing
/// the connection instead leaves the acknowledged file in place.
pub fn retain_script(name: &str, len: u64) -> String {
    format!(
        "{}\
        {}\
        head -c {len} >&3 || exit 1\n\
        exec 3>&-\n\
        {}\
        printf '%s\\n' \"$file\" || exit 1\n\
        IFS= read -r decision || exit 1\n\
        [ \"$decision\" = {RETAIN} ] || exit 1\n\
        complete=1\n\
        printf '{RETAINED}\\n' || exit 1\n\
        if IFS= read -r decision && [ \"$decision\" = {DISCARD} ]; then\n\
            cleanup\n\
        fi\n",
        staging_directory(),
        create_exclusively(name),
        verify_size(len),
    )
}

/// The lines every staging script opens with: a private umask, the removal
/// `cleanup` that runs at exit unless `$complete` says the client kept the
/// file, and `$dir` naming an empty staging directory minted by `mktemp`
/// under the host's physical temporary directory.
fn staging_directory() -> String {
    format!(
        r#"umask 077
dir=
file=
complete=0
cleanup() {{
    if [ -n "$file" ]; then rm -f -- "$file"; fi
    rmdir -- "$dir"
}}
trap 'if [ "$complete" != 1 ] && [ -n "$dir" ]; then cleanup; fi' 0
trap 'exit 1' HUP INT TERM
base=$(CDPATH= cd -P -- "${{TMPDIR:-/tmp}}" && pwd -P) || exit 1
dir=$(mktemp -d "${{base%/}}/{DIRECTORY}") || exit 1
"#
    )
}

/// The exclusive create of the script's destination, and the `$file` the exit
/// trap may remove only once the host's own create has succeeded.
fn create_exclusively(name: &str) -> String {
    format!(
        "destination=\"$dir\"/{}\n\
        set -C\n\
        exec 3> \"$destination\" || exit 1\n\
        file=$destination\n",
        shell_quote(name)
    )
}

/// The byte count `$file` must hold before the client is told about it. A
/// short transfer is never offered, and the exit trap removes its remnants.
fn verify_size(len: u64) -> String {
    format!(
        "bytes=$(wc -c < \"$file\" | tr -d '[:space:]') || exit 1\n\
        [ \"$bytes\" = {len} ] || exit 1\n"
    )
}

/// The staging directory a host reported a file inside, when `path` has a
/// shape a script can print: a bounded absolute path with no control
/// characters, no empty, `.` or `..` component, and a last two components of
/// a staging directory and a plain file name.
///
/// Shape alone is not proof the path is the host's: it decides what may be
/// pasted into a terminal, never what a remote account may be trusted with.
pub fn staged_directory_of(path: &str) -> Option<&str> {
    staged_parts(path).map(|(dir, _)| dir)
}

/// Whether `path` is `name` inside a staging directory, exactly as the staging
/// script reported it.
pub fn is_staged_file(path: &str, name: &str) -> bool {
    staged_parts(path).is_some_and(|(_, file)| file == name)
}

fn staged_parts(path: &str) -> Option<(&str, &str)> {
    if path.len() > LINE_LIMIT || path.chars().any(char::is_control) {
        return None;
    }
    // Split the raw POSIX spelling, not Path::components which normalizes away
    // repeated separators and '.', and has host-specific behavior on Windows.
    let (dir, name) = path.rsplit_once('/')?;
    (valid_directory(dir) && !matches!(name, "" | "." | "..")).then_some((dir, name))
}

/// Whether `dir` is a staging directory a script can have minted.
pub fn valid_directory(dir: &str) -> bool {
    if !dir.starts_with('/') || dir.len() > LINE_LIMIT || dir.chars().any(char::is_control) {
        return false;
    }
    if dir[1..].split('/').any(|s| matches!(s, "" | "." | "..")) {
        return false;
    }
    dir.rsplit('/')
        .next()
        .and_then(|s| s.strip_prefix(PREFIX))
        .is_some_and(|s| s.len() == SUFFIX && s.bytes().all(|b| b.is_ascii_alphanumeric()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests;
