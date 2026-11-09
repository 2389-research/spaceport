// ABOUTME: Checks trusted agent session references and preserved launch flags.
// ABOUTME: Keeps original pure helper regressions after migration removal.
#![allow(clippy::unwrap_used)]

use super::*;

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

fn session(agent: &str, kind: SessionKind, value: &str) -> AgentSession {
    AgentSession {
        agent: agent.into(),
        kind,
        source: format!("herdr:{agent}"),
        value: value.into(),
    }
}

#[test]
fn sessions_are_accepted_only_from_herdr_with_safe_references() {
    let accepts = |agent: &str, source: &str, kind: SessionKind, value: &str| {
        let mut reported = session(agent, kind, value);
        reported.source = source.into();
        AgentKind::parse(agent).unwrap().accepts(&reported)
    };
    assert!(accepts(
        "agy",
        "herdr:antigravity_cli",
        SessionKind::Id,
        "a"
    ));
    assert!(!accepts("agy", "herdr:agy", SessionKind::Id, "a"));
    assert!(accepts("cursor", "herdr:cursor", SessionKind::Id, "a"));
    assert!(accepts(
        "letta",
        "herdr:letta",
        SessionKind::Id,
        "default:agent-1"
    ));
    assert!(!accepts(
        "letta",
        "herdr:letta",
        SessionKind::Id,
        "default:"
    ));
    assert!(!accepts("copilot", "herdr:copilot", SessionKind::Id, ""));
    assert!(!accepts(
        "copilot",
        "herdr:copilot",
        SessionKind::Id,
        "--yolo"
    ));
    assert!(!accepts(
        "copilot",
        "herdr:copilot",
        SessionKind::Id,
        "a\nb"
    ));
    assert!(accepts(
        "copilot",
        "herdr:copilot",
        SessionKind::Id,
        &"a".repeat(512)
    ));
    assert!(!accepts(
        "copilot",
        "herdr:copilot",
        SessionKind::Id,
        &"a".repeat(513)
    ));
    // Only pi and omp report session files, and only absolute ones.
    assert!(!accepts(
        "copilot",
        "herdr:copilot",
        SessionKind::Path,
        "/s"
    ));
    assert!(accepts("omp", "herdr:omp", SessionKind::Path, "/h/s.jsonl"));
    assert!(!accepts("omp", "herdr:omp", SessionKind::Path, "s.jsonl"));
    assert!(accepts("pi", "herdr:pi", SessionKind::Id, "01a0"));
}

#[test]
fn copilot_flags_keep_their_values() {
    assert_eq!(
        AgentKind::Copilot.kept_flags(&strings(&[
            "copilot",
            "--model",
            "gpt-5",
            "--allow-tool",
            "shell(git)",
            "write",
            "--alt-screen",
            "off",
            "--resume",
            "old",
            "--share=/tmp/s.md",
            "-i",
            "fix it",
            "--log-dir",
            "/tmp/l",
            "--continue",
            "--yolo",
        ])),
        strings(&[
            "--model",
            "gpt-5",
            "--allow-tool",
            "shell(git)",
            "write",
            "--alt-screen",
            "off",
            "--yolo"
        ])
    );
    // Flags of agents whose options are unknown are never replayed.
    assert!(
        AgentKind::Letta
            .kept_flags(&strings(&["letta", "--agent", "a", "--yolo"]))
            .is_empty()
    );
}

#[test]
fn kept_flags_drop_sessions_positionals_and_directories() {
    assert_eq!(
        AgentKind::Codex.kept_flags(&strings(&[
            "codex",
            "-C",
            "/src",
            "--cd=/src",
            "-m",
            "o3",
            "resume",
            "--yolo",
            "hi"
        ])),
        strings(&["-m", "o3", "--yolo"])
    );
    assert_eq!(
        AgentKind::Opencode.kept_flags(&strings(&[
            "opencode",
            "/src",
            "--session",
            "s",
            "-c",
            "--model=a/b"
        ])),
        strings(&["--model=a/b"])
    );
    assert!(
        AgentKind::Claude
            .kept_flags(&strings(&["claude", "--model"]))
            .is_empty()
    );
}

#[test]
fn every_kind_is_listed_once_and_parses_back() {
    for (index, kind) in AgentKind::ALL.iter().enumerate() {
        assert_eq!(AgentKind::parse(kind.name()), Some(*kind));
        assert!(!AgentKind::ALL[..index].contains(kind), "{kind:?}");
    }
}
