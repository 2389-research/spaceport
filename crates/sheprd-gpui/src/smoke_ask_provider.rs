// ABOUTME: Qualifies native Ask input, real observation tools and exact source navigation.
// ABOUTME: Requires explicit real-provider opt-in and uses only the harness's private daemon.
use super::*;
use crate::sidebar::native_tests::{HostArrows, Target, host_arrow};
use sheprd_chat_protocol::{EntryKind, ToolContent, ToolStatus, Transcript};
use sheprd_control_protocol::protocol::observation::{ExcerptKind, Reply as Observation};

#[cfg(test)]
#[path = "smoke_ask_provider/tests.rs"]
mod tests;

pub(crate) fn enabled() -> bool {
    std::env::var("HERDR_TEST_ASK_PROVIDER").as_deref() == Ok("1")
        && std::env::var("SPACEPORT_TEST_REAL_PROVIDER").as_deref() == Ok("1")
}

pub(crate) fn control(id: String, button: Stateful<Div>) -> Stateful<Div> {
    if !enabled() {
        return button;
    }
    button
        .relative()
        .child(div().absolute().inset_0().child(host_arrow(id)))
}

pub(super) fn start(handle: WindowHandle<SheprdWindow>, cx: &mut App) {
    cx.spawn(async move |cx| {
        let result = verify(handle,cx).await;
        match result {
            Ok(()) => { EXIT_CODE.store(0,Ordering::SeqCst); eprintln!("ASK provider native PASS: real controller/helper/Pi; terminal observation; exact source and return; native multiline input; handler composition"); }
            Err(error) => { EXIT_CODE.store(1,Ordering::SeqCst); eprintln!("ASK provider native FAIL: {error:#}"); }
        }
        // The actual controller's quit hook checkpoints and reaps its helper and Pi.
        cx.update(|cx|cx.quit());
    }).detach();
}

fn inspect<T>(
    handle: WindowHandle<SheprdWindow>,
    cx: &mut AsyncApp,
    read: impl FnOnce(&Entity<SheprdWindow>, &mut Window, &mut App) -> Result<T>,
) -> Result<T> {
    AnyWindowHandle::from(handle).update(cx, |root, window, cx| {
        let view = root
            .downcast::<SheprdWindow>()
            .map_err(|_| anyhow!("unexpected native fixture root"))?;
        window.refresh();
        window.draw(cx).clear(cx);
        read(&view, window, cx)
    })?
}

async fn until<T>(
    handle: WindowHandle<SheprdWindow>,
    cx: &mut AsyncApp,
    label: &str,
    timeout: Duration,
    mut read: impl FnMut(&Entity<SheprdWindow>, &mut Window, &mut App) -> Result<Option<T>>,
) -> Result<T> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(value) = inspect(handle, cx, |view, window, cx| read(view, window, cx))? {
            return Ok(value);
        }
        if Instant::now() >= deadline {
            bail!("native provider fixture timed out: {label}");
        }
        cx.background_executor()
            .timer(Duration::from_millis(50))
            .await;
    }
}

async fn ready(
    handle: WindowHandle<SheprdWindow>,
    cx: &mut AsyncApp,
    pane: Option<&str>,
) -> Result<(String, String)> {
    until(
        handle,
        cx,
        &format!("private terminal {pane:?} readiness"),
        Duration::from_secs(20),
        |view, _, cx| {
            let view = view.read(cx);
            if let Some(error) = view.local_error.as_ref().or(view.live.error.as_ref()) {
                bail!("private daemon: {error}");
            }
            let Some(snapshot) = view.live.snapshot.as_ref() else {
                return Ok(None);
            };
            let Some(surface) = view.live.surface.as_ref() else {
                return Ok(None);
            };
            if !view.input_ready()
                || view.activation_deadline.is_some()
                || snapshot.boot_id != surface.boot_id
                || snapshot.revision != surface.projection_revision
            {
                return Ok(None);
            }
            let Some(focused) = snapshot.focused_pane_id.as_ref() else {
                return Ok(None);
            };
            if pane.is_some_and(|expected| focused != expected) || !shell_ready(&surface.frame) {
                return Ok(None);
            }
            let workspace = snapshot
                .focused_workspace_id
                .clone()
                .context("no private workspace")?;
            Ok(Some((workspace, focused.clone())))
        },
    )
    .await
}

async fn tab_from_ask(
    handle: WindowHandle<SheprdWindow>,
    cx: &mut AsyncApp,
    from: &str,
    to: &str,
    command: Command,
) -> Result<()> {
    inspect(handle, cx, |view, window, cx| {
        view.update(cx, |view, cx| {
            view.show_activity_panel(sidebar::ActivityPanel::Workspaces, window, cx)
        });
        window.focus(&view.read(cx).focus.clone(), cx);
        Ok(())
    })?;
    ready(handle, cx, Some(from)).await?;
    inspect(handle, cx, |_, window, cx| {
        window.dispatch_action(Box::new(RunCommand { command }), cx);
        Ok(())
    })?;
    ready(handle, cx, Some(to)).await?;
    inspect(handle, cx, |view, window, cx| {
        view.update(cx, |view, cx| {
            view.show_activity_panel(sidebar::ActivityPanel::Ask, window, cx)
        });
        Ok(())
    })?;
    Ok(())
}

async fn click(handle: WindowHandle<SheprdWindow>, cx: &mut AsyncApp, id: &str) -> Result<()> {
    let (target, point) = inspect(handle, cx, |_, window, cx| {
        let point = cx
            .try_global::<HostArrows>()
            .and_then(|bounds| bounds.0.get(id))
            .context("native Ask control was not painted")?
            .center();
        Ok((Target::acquire(window)?, point))
    })?;
    eprintln!(
        "ASK provider native CLICK: {id} x={} y={} key_window={}",
        point.x.to_f64(),
        point.y.to_f64(),
        target.is_key()
    );
    target.click(point.x.to_f64(), point.y.to_f64())?;
    Ok(())
}

fn text(text: &str, window: &mut Window, cx: &mut App) -> Result<()> {
    for ch in text.chars() {
        if !window.dispatch_keystroke(
            Keystroke {
                modifiers: Modifiers::default(),
                key: ch.to_string(),
                key_char: Some(ch.to_string()),
            },
            cx,
        ) {
            bail!("native character was not handled");
        }
    }
    Ok(())
}

async fn capture(handle: WindowHandle<SheprdWindow>, cx: &mut AsyncApp, name: &str) -> Result<()> {
    let Some(root) = std::env::var_os("HERDR_TEST_ASK_PROVIDER_CAPTURE") else {
        return Ok(());
    };
    let path = std::path::PathBuf::from(root).join(format!("ask-provider-{name}.png"));
    if !path.is_absolute() {
        bail!("provider capture root must be absolute");
    }
    let pixels = inspect(handle, cx, |_, window, _| window.render_to_image())?;
    cx.background_spawn(async move { pixels.save(path) })
        .await?;
    Ok(())
}

fn transcript(cx: &App) -> Result<Transcript> {
    crate::ask::fixture_transcript(cx).context("real Ask controller is missing")
}

fn completed_provider_turn(transcript: &Transcript) -> Result<bool> {
    let end = transcript.entries.iter().rev().find_map(|entry| {
        if let EntryKind::Turn(turn) = &entry.kind {
            Some(turn.end)
        } else {
            None
        }
    });
    match end {
        None => return Ok(false),
        Some(sheprd_chat_protocol::TurnEnd::Done) => return Ok(true),
        Some(sheprd_chat_protocol::TurnEnd::Failed) => {}
        Some(end) => bail!("actual provider turn did not complete successfully: {end:?}"),
    }
    if let Some(error) = transcript.entries.iter().rev().find_map(|entry| {
        if let EntryKind::Notice {
            level: sheprd_chat_protocol::NoticeLevel::Error,
            text,
            ..
        } = &entry.kind
        {
            Some(text)
        } else {
            None
        }
    }) {
        bail!("actual provider turn failed: {error}");
    }
    bail!("actual provider turn failed without a diagnostic");
}

async fn verify(handle: WindowHandle<SheprdWindow>, cx: &mut AsyncApp) -> Result<()> {
    let (workspace, first) = ready(handle, cx, None).await?;
    let value = format!("native-observed-{}", uuid::Uuid::new_v4().simple());
    let marker = format!("NATIVE_RESULT={value}");
    inspect(handle, cx, |view, window, cx| {
        window.focus(&view.read(cx).focus.clone(), cx);
        text(
            &format!("printf '\\nNATIVE_RESULT=%s\\n' '{value}'"),
            window,
            cx,
        )?;
        key("enter", window, cx)
    })?;
    until(
        handle,
        cx,
        "synthetic terminal output",
        Duration::from_secs(10),
        |view, _, cx| {
            Ok(view
                .read(cx)
                .live
                .surface
                .as_ref()
                .filter(|surface| has_output(&surface.frame, &marker))
                .map(|_| ()))
        },
    )
    .await?;
    inspect(handle, cx, |_, window, cx| {
        window.dispatch_action(
            Box::new(RunCommand {
                command: Command::Tab,
            }),
            cx,
        );
        Ok(())
    })?;
    let (_, second) = until(
        handle,
        cx,
        "second private terminal",
        Duration::from_secs(15),
        |view, _, cx| {
            let state = view.read(cx);
            let Some(snapshot) = state.live.snapshot.as_ref() else {
                return Ok(None);
            };
            Ok(snapshot
                .focused_pane_id
                .as_ref()
                .filter(|pane| *pane != &first)
                .map(|pane| (workspace.clone(), pane.clone())))
        },
    )
    .await?;
    ready(handle, cx, Some(&second)).await?;
    inspect(handle, cx, |_, window, cx| {
        window.dispatch_action(
            Box::new(RunCommand {
                command: Command::PreviousTab,
            }),
            cx,
        );
        Ok(())
    })?;
    ready(handle, cx, Some(&first)).await?;
    inspect(handle, cx, |view, window, cx| {
        view.update(cx, |view, cx| {
            view.show_activity_panel(sidebar::ActivityPanel::Ask, window, cx)
        });
        Ok(())
    })?;
    until(
        handle,
        cx,
        "idle real Ask controller",
        Duration::from_secs(10),
        |_, _, cx| {
            let status = crate::ask::status(cx).context("actual controller not installed")?;
            if status.error.is_some() {
                bail!("Ask startup: {:?}", status.error);
            }
            Ok((!status.loading
                && !status.configured
                && !status.answering
                && !status.history_present)
                .then_some(()))
        },
    )
    .await?;
    tab_from_ask(handle, cx, &first, &second, Command::NextTab).await?;
    tab_from_ask(handle, cx, &second, &first, Command::PreviousTab).await?;
    eprintln!(
        "ASK provider native PRECHECK PASS: staged Ask/workspace/tab navigation before provider submission"
    );
    capture(handle, cx, "first-use").await?;
    click(handle, cx, "ask-choose-pi").await?;
    until(
        handle,
        cx,
        "native Pi selection",
        Duration::from_secs(10),
        |_, _, cx| {
            if let Some(error) = crate::ask::status(cx).and_then(|status| status.error) {
                bail!("native Pi selection: {error}");
            }
            Ok(crate::ask::status(cx)
                .filter(|state| state.configured && !state.loading)
                .map(|_| ()))
        },
    )
    .await?;
    let before = inspect(handle, cx, |view, _, cx| Ok(view.read(cx).input_probe))?;
    inspect(handle, cx, |view, window, cx| {
        let ask = view.read(cx).ask.clone();
        text("Authorized isolated native qualification. ", window, cx)?;
        if !ask.update(cx, |ask, cx| ask.fixture_mark_composition(window, cx)) {
            bail!("native input handler marking failed");
        }
        Ok(())
    })?;
    let question = inspect(handle, cx, |view, window, cx| {
        let ask = view.read(cx).ask.clone();
        let marked = ask
            .update(cx, |ask, cx| ask.fixture_composition_bounds(window, cx))
            .context("native UTF-16 composition bounds")?;
        let composer = cx
            .try_global::<HostArrows>()
            .and_then(|bounds| bounds.0.get("ask-provider-composer"))
            .context("painted native composer")?;
        if !composer.contains(&marked.origin)
            || marked.right() > composer.right()
            || marked.bottom() > composer.bottom()
        {
            bail!("native composition bounds escaped the composer");
        }
        window.dispatch_keystroke(crate::keymap::parse_keystroke("enter")?, cx);
        if crate::ask::status(cx).is_some_and(|status| status.answering)
            || !transcript(cx)?.entries.is_empty()
        {
            bail!("Enter submitted while composing");
        }
        if !ask.update(cx, |ask, cx| ask.fixture_commit_composition(window, cx)) {
            bail!("native input handler commit failed");
        }
        key("shift-enter", window, cx)?;
        text(
            "Use spaceport_list_sessions and spaceport_read_terminal to find the terminal containing a final NATIVE_RESULT line. Quote that exact line and cite its returned source target as a spaceport-source Markdown link. The result value is deliberately absent from this question. Use only the read-only Spaceport tools. Keep the answer under 100 words.",
            window,
            cx,
        )?;
        let draft = ask.read(cx).fixture_draft(cx);
        if !draft.contains("観測🛸\nUse spaceport_list_sessions") {
            bail!("native multiline composition was not retained");
        }
        key("enter", window, cx)?;
        if !ask.read(cx).fixture_draft(cx).is_empty() {
            bail!("plain Enter did not submit the native draft");
        }
        Ok(draft)
    })?;
    let target=until(handle,cx,"provider answer with actual terminal evidence",Duration::from_secs(180),|view,_,cx| {
        let status=crate::ask::status(cx).context("Ask controller disappeared")?;
        if let Some(error)=status.error {bail!("actual Ask request failed: {error}");}
        let transcript=transcript(cx)?;
        if status.answering || status.loading {return Ok(None);}
        if !completed_provider_turn(&transcript)? {return Ok(None);}
        let users:Vec<_>=transcript.entries.iter().filter_map(|entry|if let EntryKind::User{text,..}=&entry.kind {Some(text)} else{None}).collect();
        if users.len()!=1 || !users[0].contains(&question) {bail!("native request was duplicated or lost");}
        let mut target=None;
        let mut listed=false;
        for entry in &transcript.entries {
            if let EntryKind::Tool(tool)=&entry.kind {
                if !sheprd_chat_protocol::ask::TOOLS.contains(&tool.title.as_str()) {bail!("unexpected executable tool");}
                if tool.title=="spaceport_list_sessions" && tool.status==ToolStatus::Completed {listed=true;}
                for content in &tool.content {
                    if let ToolContent::Text{text} =content && let Ok(Observation::Excerpt{source,kind:ExcerptKind::Terminal,text,observed_at_ms,..})=serde_json::from_str::<Observation>(text) && text.contains(&marker) && observed_at_ms>0 {target=Some(source.target);}
                }
            }
        }
        let Some(target)=target else{bail!("provider finished without the real synthetic terminal excerpt");};
        if !listed || !transcript.entries.iter().any(|entry|matches!(&entry.kind,EntryKind::Agent{text} if text.contains(&marker) && text.contains(&format!("spaceport-source:{}",target.0)))) {bail!("answer omitted observed marker or exact source citation");}
        let probe=view.read(cx).input_probe;
        if probe.text!=before.text || probe.keys!=before.keys || probe.actions!=before.actions {bail!("Ask native input reached hidden terminal");}
        Ok(Some(target))
    }).await?;
    capture(handle, cx, "answer").await?;
    tab_from_ask(handle, cx, &first, &second, Command::NextTab).await?;
    click(handle, cx, &format!("ask-source-target-{}", target.0)).await?;
    until(
        handle,
        cx,
        "exact observed source navigation",
        Duration::from_secs(15),
        |view, _, cx| {
            let view = view.read(cx);
            Ok((view.activity.panel == sidebar::ActivityPanel::Workspaces
                && view.live.snapshot.as_ref().is_some_and(|snapshot| {
                    snapshot.focused_pane_id.as_deref() == Some(first.as_str())
                }))
            .then_some(()))
        },
    )
    .await?;
    capture(handle, cx, "source").await?;
    inspect(handle, cx, |_, window, cx| {
        window.dispatch_action(
            Box::new(RunCommand {
                command: Command::Back,
            }),
            cx,
        );
        Ok(())
    })?;
    inspect(handle, cx, |view, _, cx| {
        if view.read(cx).activity.panel != sidebar::ActivityPanel::Ask {
            bail!("Back did not return to Ask");
        }
        if transcript(cx)?
            .entries
            .iter()
            .filter(|entry| matches!(entry.kind, EntryKind::User { .. }))
            .count()
            != 1
        {
            bail!("source return submitted another question");
        }
        Ok(())
    })?;
    capture(handle, cx, "return").await?;
    eprintln!(
        "ASK provider native EVIDENCE: source_window={} exact_pane={} different_pane={} source_target={} one_question=true observed_marker=true provider_configuration_unchanged=true handler_composition_only=true",
        handle.window_id().as_u64(),
        first,
        second,
        target.0
    );
    Ok(())
}
