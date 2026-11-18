// ABOUTME: Observes a real release window only when its private profile harness requests it.
// ABOUTME: Leaves rendering and scheduling unchanged while waiting off the UI thread.
pub(crate) mod report;

use crate::SheprdWindow;
use anyhow::{Context as _, Result, ensure};
use futures_channel::oneshot;
use gpui::{App, AppContext as _, WindowHandle};
use report::{MAX_LINE, OBSERVER_ENV, Observation, Reply, Request, SOCKET};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    os::unix::{
        fs::PermissionsExt,
        net::{UnixListener, UnixStream},
    },
    path::PathBuf,
    time::Duration,
};

pub(crate) fn start(handle: WindowHandle<SheprdWindow>, cx: &mut App) {
    let Some(directory) = std::env::var_os(OBSERVER_ENV).map(PathBuf::from) else {
        return;
    };
    let (send, mut receive) = oneshot::channel();
    let server_directory = directory.clone();
    if let Err(error) = std::thread::Builder::new()
        .name("working-profile".into())
        .spawn(move || {
            if let Err(error) = serve(server_directory, send) {
                eprintln!("WORKING PROFILE observer failed: {error:#}");
            }
        })
    {
        eprintln!("WORKING PROFILE observer thread failed: {error}");
        return;
    }
    cx.spawn(async move |cx| {
        let outcome: Result<()> = async {
            loop {
                let Incoming {
                    stream,
                    request,
                    next,
                } = receive.await.context("profile observer stopped")?;
                receive = next;
                let Request::Observe { id, capture } = request else {
                    cx.background_spawn(async move { answer(stream, &Reply::Stopped) })
                        .await?;
                    break;
                };
                let (observation, image) = handle.update(cx, |view, window, cx| {
                    let target = crate::sidebar::native_tests::Target::acquire(window)?;
                    let main = objc2::MainThreadMarker::new()
                        .context("profile activation observation requires the main thread")?;
                    let app = objc2_app_kit::NSApplication::sharedApplication(main);
                    let observation = Observation {
                        request_id: id,
                        gui_pid: std::process::id(),
                        release: !cfg!(debug_assertions),
                        window_id: window.window_handle().window_id().as_u64(),
                        window_active: window.is_window_active(),
                        window_visible: window.is_visible(),
                        native_key: target.is_key(),
                        app_active: app.isActive(),
                        app_hidden: app.isHidden(),
                        native_activation: target.activation_facts()?,
                        workspace: view
                            .live
                            .snapshot
                            .as_ref()
                            .and_then(|snapshot| snapshot.focused_workspace_id.clone()),
                        synthetic_working_panes: view.live.snapshot.as_ref().map_or_else(
                            Vec::new,
                            |snapshot| {
                                snapshot
                                    .agents
                                    .iter()
                                    .filter(|agent| {
                                        agent.agent_status
                                            == sheprd_client::protocol::AgentStatus::Working
                                    })
                                    .map(|agent| agent.pane_id.clone())
                                    .collect()
                            },
                        ),
                        groups: view.working_profile_groups(cx),
                    };
                    let image = capture.then(|| window.render_to_image()).transpose()?;
                    anyhow::Ok((observation, image))
                })??;
                let output = directory.clone();
                cx.background_spawn(async move {
                    if let Some(image) = image {
                        image.save(output.join(format!("capture-{id}.png")))?;
                        fs::write(
                            output.join(format!("observation-{id}.json")),
                            serde_json::to_vec_pretty(&observation)?,
                        )?;
                    }
                    answer(stream, &Reply::Observation(observation))
                })
                .await?;
            }
            Ok(())
        }
        .await;
        if let Err(error) = outcome {
            eprintln!("WORKING PROFILE observer failed: {error:#}");
        }
    })
    .detach();
}

fn answer(mut stream: UnixStream, reply: &Reply) -> Result<()> {
    let bytes = serde_json::to_vec(reply)?;
    ensure!(
        bytes.len() as u64 <= MAX_LINE,
        "profile observation exceeds bound"
    );
    stream.write_all(&bytes)?;
    stream.write_all(b"\n")?;
    Ok(())
}

struct Incoming {
    stream: UnixStream,
    request: Request,
    next: oneshot::Receiver<Incoming>,
}

fn serve(directory: PathBuf, mut send: oneshot::Sender<Incoming>) -> Result<()> {
    ensure!(
        directory.is_absolute(),
        "profile directory must be absolute"
    );
    let metadata = fs::metadata(&directory)?;
    ensure!(
        metadata.is_dir() && metadata.permissions().mode() & 0o077 == 0,
        "profile directory must be private"
    );
    let listener = UnixListener::bind(directory.join(SOCKET)).context("bind profile observer")?;
    let mut captures = 0;
    for _ in 0..512 {
        // The dedicated thread blocks here; it occupies no GPUI worker or UI timer.
        let (stream, _) = listener.accept()?;
        stream.set_read_timeout(Some(Duration::from_secs(15)))?;
        stream.set_write_timeout(Some(Duration::from_secs(15)))?;
        let mut line = String::new();
        BufReader::new(stream.try_clone()?)
            .take(MAX_LINE + 1)
            .read_line(&mut line)?;
        ensure!(
            line.len() as u64 <= MAX_LINE && line.ends_with('\n'),
            "invalid profile request size"
        );
        let request: Request = serde_json::from_str(&line)?;
        if matches!(request, Request::Observe { capture: true, .. }) {
            captures += 1;
            ensure!(captures <= 8, "profile capture limit exceeded");
        }
        let stop = matches!(request, Request::Stop);
        let (next_send, next) = oneshot::channel();
        if send
            .send(Incoming {
                stream,
                request,
                next,
            })
            .is_err()
            || stop
        {
            return Ok(());
        }
        send = next_send;
    }
    anyhow::bail!("profile observation request limit exceeded")
}
