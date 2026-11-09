//! Checkout identity used by the remaining workspace actions.

/// The main checkout a `<root>/.git` Git directory belongs to. Any other key
/// (a bare repository, a separate Git directory) does not name its checkout.
pub(super) fn checkout_root(key: &str) -> Option<&str> {
    key.strip_suffix("/.git")
        .or_else(|| key.strip_suffix("\\.git"))
        .filter(|root| !root.is_empty())
}

/// Whether `cwd` lies outside the main checkout whose Git directory is `key`.
/// This compares endpoint paths rather than reading local files. Either
/// separator is accepted, and Windows paths ignore case.
pub(super) fn left_checkout(key: &str, cwd: &str) -> bool {
    let Some(root) = checkout_root(key) else {
        return false;
    };
    if cwd.is_empty() {
        return false;
    }
    let windows = !root.starts_with('/');
    let normalize = |path: &str| {
        let path = path.trim_end_matches(['/', '\\']);
        if windows {
            path.replace('\\', "/").to_lowercase()
        } else {
            path.to_owned()
        }
    };
    let (root, cwd) = (normalize(root), normalize(cwd));
    let inside = cwd == root
        || cwd
            .strip_prefix(&root)
            .is_some_and(|rest| rest.starts_with('/'));
    !inside
}
