// ABOUTME: Locates configuration, tools, and isolated application state.
// ABOUTME: Separates compiled build namespaces from platform root discovery.
//! Where this user's files and tools live, for an app started from the
//! desktop rather than a shell: the home directory, the configuration root,
//! Spaceport's state directory, and a `PATH` with the directories a GUI launch
//! leaves out.
use std::{ffi::OsString, path::PathBuf};

pub const PREVIEW_STATE_NAMESPACE: &str = "spaceport";

/// A stable, bounded state name from the application's compiled identity.
/// Preview versions share state; development profiles and worktrees stay apart.
pub fn state_namespace(preview: bool, profile: &str, worktree: Option<&str>) -> Option<String> {
    if preview {
        return Some(PREVIEW_STATE_NAMESPACE.to_owned());
    }
    if !valid_namespace(profile) || profile.len() > 32 {
        return None;
    }
    let mut name = format!("spaceport-development-{profile}");
    if let Some(worktree) = worktree {
        if worktree.is_empty() {
            return None;
        }
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in worktree.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
        name.push_str(&format!("-{hash:016x}"));
    }
    Some(name)
}

fn valid_namespace(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

// Probing Linuxbrew on macOS triggers /home's automounter and directory lookups.
pub const HOMEBREW_DIRS: &[&str] = if cfg!(target_os = "linux") {
    &[
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/home/linuxbrew/.linuxbrew/bin",
    ]
} else if cfg!(unix) {
    &["/opt/homebrew/bin", "/usr/local/bin"]
} else {
    &[]
};

/// Why the configuration root is unknown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PathError {
    #[error("HOME is not set")]
    MissingHome,
    #[error("XDG_CONFIG_HOME must be an absolute path")]
    RelativeConfigRoot,
}

/// Where configuration lives: `XDG_CONFIG_HOME` (also as the login shell
/// sets it, see [`crate::local_var`]), else `%APPDATA%` on Windows, else
/// `~/.config`. Herdr's own files are under its `herdr` directory.
pub fn config_root() -> Result<PathBuf, PathError> {
    match crate::local_var("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
        Some(value) => {
            let path = PathBuf::from(value);
            if !path.is_absolute() {
                return Err(PathError::RelativeConfigRoot);
            }
            Ok(path)
        }
        None => {
            #[cfg(windows)]
            if let Some(roaming) = std::env::var_os("APPDATA").filter(|value| !value.is_empty()) {
                return Ok(PathBuf::from(roaming));
            }
            let home = home_dir().ok_or(PathError::MissingHome)?;
            #[cfg(windows)]
            return Ok(home.join("AppData").join("Roaming"));
            #[cfg(not(windows))]
            Ok(home.join(".config"))
        }
    }
}

/// This user's home directory: `HOME`, or `USERPROFILE` on Windows. None
/// when neither is set to something non-empty.
pub fn home_dir() -> Option<PathBuf> {
    let variable = |name| std::env::var_os(name).filter(|value: &OsString| !value.is_empty());
    variable("HOME")
        .or_else(|| {
            if cfg!(windows) {
                variable("USERPROFILE")
            } else {
                None
            }
        })
        .map(PathBuf::from)
}

/// Spaceport's own state directory, shared by its preferences, logs, and
/// saved devices.
pub fn state_dir() -> Option<PathBuf> {
    state_dir_for(PREVIEW_STATE_NAMESPACE)
}

/// This user's state directory within one validated application namespace.
pub fn state_dir_for(namespace: &str) -> Option<PathBuf> {
    valid_namespace(namespace).then_some(())?;
    state_root_with(|name| std::env::var_os(name)).map(|root| root.join(namespace))
}

#[cfg(test)]
fn state_dir_with(var: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    state_root_with(var).map(|root| root.join(PREVIEW_STATE_NAMESPACE))
}

fn state_root_with(var: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    state_root_for(var, cfg!(windows))
}

fn state_root_for(var: impl Fn(&str) -> Option<OsString>, windows: bool) -> Option<PathBuf> {
    let var = |name| var(name).filter(|value| !value.is_empty());
    var("XDG_STATE_HOME").map(PathBuf::from).or_else(|| {
        // Git Bash can set a different HOME from the desktop's profile.
        // Both launches must use the same Windows state root.
        if windows {
            if let Some(local) = var("LOCALAPPDATA") {
                return Some(PathBuf::from(local));
            }
            if let Some(profile) = var("USERPROFILE") {
                return Some(PathBuf::from(profile).join("AppData").join("Local"));
            }
        }
        var("HOME").map(|home| PathBuf::from(home).join(".local").join("state"))
    })
}

/// GUI launches omit directories commonly used by user-installed tools.
pub fn local_path() -> OsString {
    let mut dirs = Vec::new();
    if let Some(home) = home_dir() {
        dirs.extend(
            [".local/bin", ".cargo/bin", ".bun/bin", ".npm-global/bin"].map(|dir| home.join(dir)),
        );
    }
    dirs.extend(HOMEBREW_DIRS.iter().copied().map(PathBuf::from));
    let inherited = std::env::var_os("PATH").unwrap_or_default();
    dirs.extend(std::env::split_paths(&inherited));
    // launchd's `PATH` stops at `/usr/bin:/bin`; system tools such as `lsof`,
    // which the Tailscale CLI runs on macOS, live in the `sbin` directories.
    if cfg!(unix) {
        for system in ["/usr/sbin", "/sbin"].map(PathBuf::from) {
            if !dirs.contains(&system) {
                dirs.push(system);
            }
        }
    }
    std::env::join_paths(dirs).unwrap_or(inherited)
}

#[cfg(all(test, unix))]
mod tests;

#[cfg(test)]
#[path = "local_path/tests/state_directory.rs"]
mod state_directory_tests;

#[cfg(test)]
#[path = "local_path/tests/windows_state.rs"]
mod windows_state_tests;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[path = "local_path/tests/state_namespace.rs"]
mod state_namespace_tests;
