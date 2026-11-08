//! Git branch validation and checkout identity used by repository actions.

const PREFIX: &str = "worktree";

/// Characters Git refuses anywhere in a branch name besides whitespace and
/// control characters.
const REFUSED: [char; 7] = ['~', '^', ':', '?', '*', '[', '\\'];
/// The ending Git refuses on any component of a branch name.
const LOCK: &str = ".lock";

/// Validate a literal branch name, without resolving checkout shorthand or
/// consulting a repository (which may live on a remote endpoint).
pub(super) fn validate_branch(branch: &str) -> crate::Result<()> {
    let invalid = branch.is_empty()
        || branch == "@"
        || branch == "HEAD"
        || branch.starts_with('-')
        || branch.ends_with('.')
        || branch.contains("..")
        || branch.contains("@{")
        || branch
            .chars()
            .any(|ch| ch <= ' ' || ch == '\x7f' || REFUSED.contains(&ch))
        || branch
            .split('/')
            .any(|part| part.is_empty() || part.starts_with('.') || part.ends_with(LOCK));
    if invalid {
        return Err(crate::Error::InvalidBranchName);
    }
    Ok(())
}

/// The daemon's generated branch shape, used for fan-out lane names.
pub(super) fn generated_branch_slug(seed: u64) -> String {
    const ADJECTIVES: [&str; 8] = [
        "brave", "calm", "clear", "green", "lucky", "quiet", "rapid", "silver",
    ];
    const NOUNS: [&str; 8] = [
        "river", "cloud", "field", "forest", "harbor", "meadow", "stone", "valley",
    ];
    let adjective = ADJECTIVES[(seed as usize) % ADJECTIVES.len()];
    let noun = NOUNS[((seed / ADJECTIVES.len() as u64) as usize) % NOUNS.len()];
    let suffix = seed & 0xffff;
    format!("{PREFIX}/{adjective}-{noun}-{suffix:04x}")
}

pub(super) use sheprd_github::checkout::{checkout_root, left_checkout};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_branch_names_follow_git_ref_rules() {
        for branch in [
            "config reload",
            "feature\tbranch",
            "feature\nbranch",
            "bad\u{7f}",
            "",
            "@",
            "HEAD",
            "-option",
            "a..b",
            "a@{b",
            "a~b",
            "a^b",
            "a:b",
            "a?b",
            "a*b",
            "a[b",
            "a\\b",
            "/a",
            "a/",
            "a//b",
            ".hidden",
            "a/.b",
            "a.lock",
            "a.lock/b",
            "a.",
        ] {
            assert!(
                matches!(
                    validate_branch(branch),
                    Err(crate::Error::InvalidBranchName)
                ),
                "{branch:?}"
            );
        }
        for branch in [
            "config-reload",
            "feature/test",
            "fix.v2",
            "a./b",
            "日本語",
            "a@b",
        ] {
            assert!(validate_branch(branch).is_ok(), "{branch:?}");
        }
    }
}
