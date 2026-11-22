//! Which tracing targets the collector keeps.

use super::*;

#[test]
fn a_root_is_kept_with_its_modules() {
    for target in ["spaceport", "spaceport::app", "spaceport::chat::link"] {
        assert!(kept_target(TARGETS, target), "{target}");
    }
}

#[test]
fn a_crate_with_the_prefix_is_kept_with_its_modules() {
    for target in [
        "spaceport_anything",
        "spaceport_anything::module",
        "spaceport_client::worker::read",
        // A crate of its own, which only shares another's first letters.
        "spaceport_clientx::y",
    ] {
        assert!(kept_target(TARGETS, target), "{target}");
    }
}

#[test]
fn other_targets_are_dropped() {
    for target in [
        "gpui",
        "gpui::x",
        // An unrelated crate must not match a workspace root or prefix.
        "sheprd",
        "sheprd::y",
        "sheprdx::y",
        "spaceportx::y",
        "helper_spaceport",
        // Only the crate, the first segment, is matched.
        "gpui::spaceport_client",
        "gpui::spaceport::app",
        "::spaceport_client",
        "",
        // Not a module path at all.
        "a hand-written target",
        "spaceport startup",
    ] {
        assert!(!kept_target(TARGETS, target), "{target}");
    }
}

#[test]
fn an_empty_prefix_keeps_only_the_roots() {
    let roots_only = Targets {
        crate_prefix: "",
        roots: &["spaceport"],
    };
    assert!(kept_target(roots_only, "spaceport::app"));
    assert!(!kept_target(roots_only, "gpui::x"));
    assert!(!kept_target(roots_only, "spaceport_client"));
}

/// A workspace crate nothing names reaches the log file: the collector takes
/// it by its prefix, so a crate added or reached only through another one
/// is never dropped without a trace.
#[test]
fn an_unlisted_workspace_crate_reaches_the_log_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("logs").join(FILE_NAME);
    let (capture, queued, counters) = sink(QUEUE);
    let lines = capture.lines.clone();
    let writer = {
        let path = path.clone();
        let counters = counters.clone();
        thread::spawn(move || write_lines(path, queued, &counters))
    };
    tracing::subscriber::with_default(subscriber(capture, TARGETS), || {
        tracing::warn!(target: "spaceport_brand_new::worker", "from a crate nobody listed");
        tracing::warn!(target: "tokio::runtime", "from a dependency");
    });
    assert!(flush_through(&lines, Duration::from_secs(10)));
    let mut tail = Tail::new(&path);
    tail.read().unwrap();
    let written: Vec<_> = tail
        .records()
        .iter()
        .map(|record| (record.target.clone(), record.message.clone()))
        .collect();
    assert_eq!(
        written,
        [(
            "spaceport_brand_new::worker".to_owned(),
            "from a crate nobody listed".to_owned()
        )]
    );
    assert_eq!(counters.dropped.load(Ordering::Relaxed), 0);
    drop(lines);
    writer.join().unwrap();
}
