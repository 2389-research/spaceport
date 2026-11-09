#![allow(clippy::expect_used)]
use super::*;

mod owner;
mod reconnect;
mod remote;
mod remote_userinfo;
mod retry;
mod workers;

#[test]
#[ignore = "requires HERDR_AVATAR_TEST_REPO and network access to GitHub avatars"]
fn loads_local_repository_owner_avatar() {
    let cwd = std::env::var("HERDR_AVATAR_TEST_REPO").expect("set a local GitHub repository path");
    let checkout = Checkout::Local(cwd.as_str());
    let owner = repo_owner(checkout, &Default::default())
        .ok()
        .flatten()
        .expect("repository has a GitHub origin");
    let mut avatars = Avatars::new();
    avatars.request(checkout, Instant::now());
    let deadline = Instant::now() + Duration::from_secs(10);
    while avatars.image(checkout).is_none() {
        avatars.poll(Instant::now());
        assert!(Instant::now() < deadline, "avatar worker timed out");
        thread::sleep(Duration::from_millis(20));
    }
    let image = avatars.image(checkout).expect("owner avatar downloaded");
    assert!(!image.bytes.is_empty());
    eprintln!("Avatar loaded for {owner}: {} bytes", image.bytes.len());
}

#[test]
fn parses_github_remotes() {
    for prefix in [
        "https://github.com/",
        "http://github.com/",
        "git@github.com:",
        "ssh://git@github.com/",
    ] {
        for repo in ["repo", "repo.git", "a_repo-1.2.git"] {
            assert_eq!(
                github_owner(&format!("{prefix}Some-Owner/{repo}")),
                Some("some-owner".to_owned())
            );
        }
    }
    assert_eq!(github_owner("https://github.com/a/r"), Some("a".into()));
}

#[test]
fn rejects_untrusted_hosts_and_malformed_paths() {
    for remote in [
        "https://github.com.evil.test/owner/repo",
        "https://github.com@evil.test/owner/repo",
        "https://evil.test/github.com/owner/repo",
        "https://github.com:443/owner/repo",
        "git@github.com.evil.test:owner/repo",
        "ssh://git@github.com.evil.test/owner/repo",
        "ssh://git@github.com:22/owner/repo",
        "file:///github.com/owner/repo",
        "https://github.com/owner",
        "https://github.com/owner/",
        "https://github.com/owner/.git",
        "https://github.com/owner/..",
        "https://github.com/owner/repo/extra",
        "https://github.com/owner/repo/",
        "https://github.com/owner/repo?query",
        "https://github.com/owner/repo#fragment",
        "https://github.com/owner/repo%2fextra",
        "https://github.com/owner/repo\\extra",
        "https://github.com/owner/repo\n",
    ] {
        assert_eq!(github_owner(remote), None, "{remote}");
    }
    // No account-name grammar is enforced: whatever GitHub names an owner,
    // an enterprise managed user included, survives the round trip.
    for owner in ["a_b", "a--b", "owner_shortcode", "-owner", "owner-", "a.b"] {
        assert_eq!(
            github_owner(&format!("https://github.com/{owner}/repo")).as_deref(),
            Some(owner),
            "{owner}"
        );
    }
    // Only what would not stay inside its own path segment is refused.
    for owner in ["", ".", "..", "a@b", "a%2fb", "a b", "a?b", "a#b", "\u{e9}"] {
        assert_eq!(
            github_owner(&format!("https://github.com/{owner}/repo")),
            None,
            "{owner}"
        );
    }
    assert!(github_owner(&format!("https://github.com/{}/repo", "a".repeat(100))).is_some());
    assert!(github_owner(&format!("https://github.com/{}/repo", "a".repeat(101))).is_none());
}

#[test]
fn accepts_only_supported_content_types() {
    for (mime, format) in [
        ("image/png", ImageFormat::Png),
        ("IMAGE/JPEG; charset=binary", ImageFormat::Jpeg),
        ("image/gif", ImageFormat::Gif),
        ("image/webp", ImageFormat::Webp),
    ] {
        assert_eq!(avatar_format(mime), Some(format));
    }
    for mime in [
        "",
        "text/html",
        "image/svg+xml",
        "image/png-extra",
        "application/octet-stream",
    ] {
        assert_eq!(avatar_format(mime), None);
    }
}

type Results = mpsc::Sender<(Checkout<String>, Lookup)>;

/// An `Avatars` whose worker ends are the test's: the request receiver and
/// the result sender.
pub(super) fn detached() -> (Avatars, mpsc::Receiver<Checkout<String>>, Results) {
    let (requests, incoming) = mpsc::sync_channel(MAX_REQUESTS);
    let (outgoing, results) = mpsc::channel();
    let avatars = Avatars {
        local_requests: requests.clone(),
        remote_requests: requests,
        results,
        local: HashMap::new(),
        remote: HashMap::new(),
        links: HashMap::new(),
        offline: links::Offline::default(),
    };
    (avatars, incoming, outgoing)
}

#[test]
fn deduplicates_pending_hits_and_misses_and_drains_results() {
    let (mut avatars, incoming, outgoing) = detached();
    let now = Instant::now();
    assert!(!avatars.poll(now));
    assert!(avatars.image(Checkout::Local("hit")).is_none());
    for cwd in ["hit", "miss", "hit", "miss"] {
        avatars.request(Checkout::Local(cwd), now);
    }
    assert_eq!(
        incoming.try_iter().collect::<Vec<_>>(),
        [
            Checkout::Local("hit".into()),
            Checkout::Local("miss".into())
        ]
    );
    let image = Arc::new(Image::empty());
    assert!(
        outgoing
            .send((Checkout::Local("hit".into()), Ok(Some(image.clone()))))
            .is_ok()
    );
    assert!(
        outgoing
            .send((Checkout::Local("miss".into()), Ok(None)))
            .is_ok()
    );
    assert!(avatars.poll(now));
    assert!(!avatars.poll(now));
    assert!(
        avatars
            .image(Checkout::Local("hit"))
            .is_some_and(|cached| Arc::ptr_eq(&cached, &image))
    );
    assert!(avatars.image(Checkout::Local("miss")).is_none());
    avatars.request(Checkout::Local("hit"), now);
    avatars.request(Checkout::Local("miss"), now);
    assert!(incoming.try_recv().is_err());
}
