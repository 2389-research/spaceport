use super::*;
use crate::{control::protocol::work::Script, menu::Page, teleport::AgentKind};
use flows::{agent_kind, agents, prompt_text};

#[test]
fn agent_names_map_case_insensitively_and_merge() {
    assert_eq!(agent_kind("Claude"), Some(AgentKind::Claude));
    assert_eq!(agent_kind("AGY"), Some(AgentKind::Antigravity));
    assert_eq!(agent_kind("antigravity"), Some(AgentKind::Antigravity));
    assert_eq!(agent_kind("cursor"), Some(AgentKind::Cursor));
    assert_eq!(agent_kind("gpt"), None);
    assert_eq!(
        agents(&[
            ("claude".into(), 2),
            ("codex".into(), 1),
            ("CLAUDE".into(), 1)
        ])
        .ok(),
        Some(vec![(AgentKind::Claude, 3), (AgentKind::Codex, 1)])
    );
    let Err(Response::Error { code, message }) = agents(&[("gpt".into(), 1)]) else {
        panic!("an unknown agent is refused");
    };
    assert_eq!(code, ErrorCode::InvalidRequest);
    assert!(
        message.contains("claude") && message.contains("codex"),
        "{message}"
    );
    assert_eq!(
        agents(&[("claude".into(), 4), ("codex".into(), 3)])
            .err()
            .as_ref()
            .and_then(error_code),
        Some(ErrorCode::InvalidRequest),
        "more lanes than a fan-out runs"
    );
}

#[test]
fn prompts_become_one_safe_line() {
    assert_eq!(
        prompt_text("  fix\nthe\tbug\u{1b}[2J \u{202e}x "),
        "fix the bug[2J x"
    );
    assert_eq!(prompt_text("\n\t"), "");
}

#[gpui::test]
fn scripts_refuse_what_the_menu_does_not_offer(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|_, cx| {
        view.update(cx, |view, cx| {
            connected(view);
            let run =
                |view: &mut SheprdWindow, workspace, kind, cx: &mut gpui::Context<SheprdWindow>| {
                    error_code(&flows::script(view, &caller(Some(workspace)), kind, cx))
                };
            assert_eq!(
                run(view, "w4", Script::Archive, cx),
                Some(ErrorCode::Unsupported),
                "archiving removes the checkout, so it stays in the delete dialog"
            );
            assert_eq!(
                run(view, "w3", Script::Setup, cx),
                Some(ErrorCode::InvalidRequest)
            );
            assert_eq!(
                run(view, "w0", Script::Run, cx),
                Some(ErrorCode::InvalidRequest)
            );
            assert!(view.worktree_script.is_none());
        })
    });
}

#[gpui::test]
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn fan_out_opens_composed_and_picks_agents_once_known(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            connected(view);
            let request =
                |agents: Vec<(String, u8)>| crate::control::protocol::work::FanOutRequest {
                    prompt: "fix\nthe flaky test".into(),
                    agents,
                };
            let refused = flows::fan_out(
                view,
                &caller(Some("w4")),
                request(vec![("claude".into(), 2)]),
                window,
                cx,
            );
            assert_eq!(
                error_code(&refused),
                Some(ErrorCode::InvalidRequest),
                "a custom socket cannot be scripted"
            );
            assert_eq!(view.menu.page, None);
            view.endpoints[0].connection.target = sheprd_client::ConnectTarget::Local;
            let opened = flows::fan_out(
                view,
                &caller(Some("w4")),
                request(vec![("claude".into(), 2), ("kimi".into(), 1)]),
                window,
                cx,
            );
            assert!(matches!(opened, Response::Done { .. }), "{opened:?}");
            assert_eq!(view.menu.page, Some(Page::FanOut));
            assert_eq!(
                view.menu.input.as_ref().map(|input| input.text.as_str()),
                Some("fix the flaky test")
            );
            let fan_out = view.fan_out.as_mut().unwrap();
            assert_eq!(
                fan_out.picked_for_test(),
                0,
                "nothing until the agents are known"
            );
            // The host lookup is replaced, so nothing depends on this machine.
            fan_out.agents_for_test(vec![AgentKind::Claude, AgentKind::Codex]);
            assert_eq!(fan_out.picked_for_test(), 2);
            assert!(fan_out.composing(), "the user starts it");
            assert_eq!(
                fan_out
                    .error_for_test()
                    .map(|error| error.replace(['\u{2068}', '\u{2069}'], ""))
                    .as_deref(),
                Some("Not installed on Local: kimi")
            );
        })
    });
}
