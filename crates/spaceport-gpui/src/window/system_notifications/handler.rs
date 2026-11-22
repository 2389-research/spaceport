//! Checks whether a Windows protocol handler points to this running binary.
//! A handler key owned by another app must not receive our notification tags.

#[cfg(test)]
mod tests;

pub(super) fn owns_handler(command: &str, executable: &std::path::Path) -> bool {
    let Some(path) = executable.to_str() else {
        return false;
    };
    command == format!("\"{path}\" \"%1\"")
}
