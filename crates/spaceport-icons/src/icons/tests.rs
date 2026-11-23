#![allow(clippy::unwrap_used)]

use super::*;
use gpui::{DevicePixels, Image, ImageFormat, TestAppContext};

#[gpui::test]
fn embedded_icons_render_nonempty_masks(cx: &mut TestAppContext) {
    let renderer = cx.update(|cx| cx.svg_renderer());
    for path in Icons.list("icons/").unwrap() {
        let bytes = Icons.load(&path).unwrap().unwrap();
        let format = if path.ends_with(".png") {
            ImageFormat::Png
        } else {
            ImageFormat::Svg
        };
        let image = Image::from_bytes(format, bytes.into_owned())
            .to_image_data(renderer.clone())
            .unwrap();
        // Square at its own scale: the tab and menu glyphs are drawn on a
        // 24px grid, GitHub's mark on its own 16px one.
        let rendered = image.size(0);
        assert_eq!(rendered.width, rendered.height, "{path}");
        assert!(rendered.width >= DevicePixels(16), "{path}");
        let pixels = image.as_bytes(0).unwrap();
        assert!(pixels.as_chunks::<4>().0.iter().any(|pixel| pixel[3] > 0));
        assert!(pixels.as_chunks::<4>().0.iter().any(|pixel| pixel[3] == 0));
    }
    assert!(Icons.load("unknown.svg").unwrap().is_none());
    // Every asset is listed once, whichever table declares it.
    let listed = Icons.list("icons/").unwrap();
    let unique: std::collections::HashSet<_> = listed.iter().collect();
    assert_eq!(unique.len(), listed.len());
    for (name, _) in STATUS_ICONS {
        assert!(listed.iter().any(|path| path.as_ref() == name), "{name}");
    }
}

/// Herdr's `agent_label` values, copied from `src/detect/mod.rs` at
/// herdrdev/herdr 65e35a38, plus the canonical native Omo identity.
const HERDR_AGENT_LABELS: &[&str] = &[
    "pi",
    "omo",
    "claude",
    "codex",
    "gemini",
    "cursor",
    "devin",
    "agy",
    "cline",
    "omp",
    "mastracode",
    "opencode",
    "copilot",
    "kimi",
    "kiro",
    "droid",
    "amp",
    "grok",
    "hermes",
    "kilo",
    "qodercli",
    "qwen",
    "letta",
    "maki",
    "muse",
];

#[test]
fn every_herdr_agent_label_has_its_own_embedded_mark() {
    let listed = Icons.list("icons/agent-").unwrap();
    let mut seen = Vec::new();
    for label in HERDR_AGENT_LABELS {
        let icon = AgentIcon::from_identity(Some(label));
        assert_ne!(icon, AgentIcon::Generic, "{label}");
        let extension = if icon == AgentIcon::Omo { "png" } else { "svg" };
        assert_eq!(icon.path(), format!("icons/agent-{label}.{extension}"));
        assert!(Icons.load(icon.path()).unwrap().is_some(), "{label}");
        assert!(listed.iter().any(|path| path == icon.path()), "{label}");
        assert!(!seen.contains(&icon), "{label}");
        seen.push(icon);
    }
    // No marks for labels Herdr does not emit, beyond the fallback.
    assert_eq!(AgentIcon::ALL.len(), HERDR_AGENT_LABELS.len() + 1);
    assert_eq!(listed.len(), AgentIcon::ALL.len());
}

#[test]
fn unknown_or_display_identities_use_the_generic_mark() {
    for identity in [
        Some("future-agent"),
        Some("Claude Code"),
        Some("kiro-cli"),
        Some("antigravity"),
        Some(""),
        None,
    ] {
        assert_eq!(AgentIcon::from_identity(identity), AgentIcon::Generic);
    }
    assert!(Icons.load(AgentIcon::Generic.path()).unwrap().is_some());
}

mod omo;
