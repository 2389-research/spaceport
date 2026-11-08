use super::*;
use std::{fs, path::Path, process::Command};

fn git(dir: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .expect("git runs");
    assert!(output.status.success(), "git {args:?}");
}

#[test]
fn scripts_list_finds_the_current_checkout_root() {
    let temp = tempfile::tempdir().unwrap();
    let main = temp.path().join("main");
    fs::create_dir_all(main.join("src")).unwrap();
    git(&main, &["init", "-q", "-b", "main"]);
    fs::write(main.join("src/a.txt"), "a\n").unwrap();
    git(&main, &["add", "-A"]);
    git(&main, &["commit", "-q", "-m", "base"]);
    let linked = temp.path().join("linked");
    git(
        &main,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feature",
            linked.to_str().unwrap(),
        ],
    );
    // From a subdirectory and a linked worktree, list scripts at each root.
    assert_eq!(
        here::root(&main.join("src")).unwrap(),
        fs::canonicalize(&main).unwrap()
    );
    assert_eq!(
        here::root(&linked).unwrap(),
        fs::canonicalize(&linked).unwrap()
    );
    let outside = tempfile::tempdir().unwrap();
    assert!(matches!(
        here::root(outside.path()),
        Err(crate::Error::NotInCheckout)
    ));
    assert!(matches!(
        rejected(crate::Error::NotInCheckout),
        crate::Error::Control(crate::control::Error::Rejected {
            code: ErrorCode::InvalidRequest,
            ..
        })
    ));
}
