// ABOUTME: Captures the native Ask page across themes, widths, and conversation states.
// ABOUTME: Verifies keyboard isolation without launching agents or reading personal sessions.
use super::*;
use sheprd_chat_protocol::{Entry, EntryId, EntryKind, Status, Transcript};

fn conversation() -> Transcript {
    let mut transcript = Transcript::new("Pi");
    transcript.status = Status::Idle;
    transcript.entries = vec![
        Entry {
            id: EntryId(1),
            kind: EntryKind::User {
                text: "What needs my attention across connected workspaces?".into(),
                files: 0,
            },
        },
        Entry {
            id: EntryId(2),
            kind: EntryKind::Agent {
                text: "Two sample sessions need attention.\n\nRelease preparation is waiting for your review. The parser test run reported a failure.\n\nSample observation: November 13, 14:32. One device was unreachable, so its sessions are not covered.\n\nLong workspace label: Spaceport session observability and native conversation presentation.".into(),
            },
        },
    ];
    transcript
}

pub(super) async fn verify(handle: WindowHandle<SheprdWindow>, cx: &mut AsyncApp) -> Result<()> {
    for light in [false, true] {
        for width in [960., 480.] {
            for configured in [false, true] {
                let transcript = configured.then(conversation);
                handle.update(cx, |view, window, cx| -> Result<()> {
                    crate::ask::fixture(transcript, cx);
                    view.ask_feature = crate::ask::Feature::current(cx);
                    view.ask = cx.new(|cx| {
                        crate::ask::View::new(view.config.clone(), view.theme.clone(), cx)
                    });
                    view.theme = if light {
                        view.config.theme_named("Catppuccin Latte", true)?
                    } else {
                        crate::config::Theme::default()
                    };
                    view.theme_light = light;
                    window.resize(size(px(width), px(720.)));
                    view.show_activity_panel(sidebar::ActivityPanel::Ask, window, cx);
                    cx.notify();
                    Ok(())
                })??;
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
                let capture = std::env::var_os("HERDR_TEST_ASK_CAPTURE");
                let pixels = AnyWindowHandle::from(handle).update(cx, |root, window, cx| -> Result<_> {
                    let view = root.downcast::<SheprdWindow>().map_err(|_| anyhow!("unexpected fixture root"))?;
                    window.refresh();
                    window.draw(cx).clear(cx);
                    let before = view.read(cx).input_probe;
                    if view.read(cx).activity.panel != sidebar::ActivityPanel::Ask {
                        bail!("Ask activity section did not open");
                    }
                    for key in ["left", "right", "backspace"] {
                        window.dispatch_keystroke(crate::keymap::parse_keystroke(key)?, cx);
                    }
                    if configured {
                        for key in ["a", "shift-enter", "b"] {
                            window.dispatch_keystroke(crate::keymap::parse_keystroke(key)?, cx);
                        }
                        let ask = view.read(cx).ask.clone();
                        if ask.read(cx).fixture_draft(cx) != "a\nb" {
                            bail!("native Ask composer did not accept text and a shifted newline");
                        }
                        window.dispatch_keystroke(crate::actions::edit_key("a").keystroke, cx);
                        window.dispatch_keystroke(crate::keymap::parse_keystroke("backspace")?, cx);
                        if !ask.read(cx).fixture_draft(cx).is_empty() {
                            bail!("native Ask composer did not clear its selected draft");
                        }
                    }
                    let state = view.read(cx);
                    if state.input_probe.text != before.text
                        || state.input_probe.keys != before.keys
                        || state.input_probe.actions != before.actions
                        || state.focus.is_focused(window)
                    {
                        bail!("Ask keyboard input reached the hidden terminal or left terminal focus active");
                    }
                    capture.as_ref().map(|_| window.render_to_image()).transpose()
                })??;
                if let (Some(root), Some(pixels)) = (capture, pixels) {
                    let root = std::path::PathBuf::from(root);
                    if !root.is_absolute() {
                        bail!("Ask capture root must be absolute");
                    }
                    let theme = if light { "light" } else { "dark" };
                    let state = if configured {
                        "conversation"
                    } else {
                        "first-use"
                    };
                    let path = root.join(format!("ask-{theme}-{width:.0}-{state}.png"));
                    cx.background_spawn(async move { pixels.save(path) })
                        .await?;
                }
            }
        }
    }
    eprintln!(
        "ASK native PASS: two themes, two widths, first use, conversation, typing, multiline drafts, and keyboard isolation"
    );
    Ok(())
}
