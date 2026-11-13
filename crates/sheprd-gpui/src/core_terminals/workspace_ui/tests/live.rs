// ABOUTME: Checks live behavior with isolated application fixtures.
// ABOUTME: Makes Core opt-in explicit while preserving retained behavior checks.
//! Tests sidebar membership against an isolated, real Core and its document CAS.
//! The fixture owns one terminal; UI clicks must leave its Core process intact.

use super::*;
use crate::core_terminals::workspace_document::edit::{self, Edit};
use gpui::{Modifiers, VisualTestContext, point, px, size};
use sheprd_core_client::{Core, CoreBackend, Endpoint, wire::Run};
use std::{
    ffi::OsString,
    io,
    path::PathBuf,
    process::{Child, Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

mod remote_selection;

struct Sandbox {
    _dir: tempfile::TempDir,
    binary: OsString,
    child: Child,
    home: PathBuf,
    socket: PathBuf,
}

impl Sandbox {
    fn start() -> Self {
        let binary = std::env::var_os("HERDR_CORE_TEST_BINARY")
            .expect("set HERDR_CORE_TEST_BINARY to an explicit private test binary");
        let parent = std::env::var_os("HERDR_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into());
        let dir = tempfile::Builder::new()
            .prefix("hcgmem")
            .tempdir_in(parent)
            .unwrap();
        let home = dir.path().join("home");
        let socket = dir.path().join("c.sock");
        let child = Self::command(&binary, &home, &socket)
            .arg("serve")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        Self {
            _dir: dir,
            binary,
            child,
            home,
            socket,
        }
    }

    fn command(binary: &OsString, home: &PathBuf, socket: &PathBuf) -> Command {
        let mut command = Command::new(binary);
        command
            .env_clear()
            .env("HC_SESSION", "membership")
            .env("HC_LAUNCHD_DIRECT", std::process::id().to_string())
            .env("HC_SOCKET", socket)
            .env("HOME", home)
            .env("TMPDIR", socket.parent().unwrap())
            .env("XDG_CONFIG_HOME", home.join("config"))
            .env("XDG_STATE_HOME", home.join("state"))
            .env("XDG_DATA_HOME", home.join("data"))
            .env("XDG_CACHE_HOME", home.join("cache"))
            .env("PATH", std::env::var_os("PATH").unwrap_or_default());
        command
    }

    fn connect(&self) -> Core {
        let endpoint = Endpoint {
            socket: self.socket.clone(),
            session: "membership".into(),
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match Core::connect(&endpoint) {
                Ok(core) => return core,
                Err(Error::Connect { .. }) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(30));
                }
                Err(error) => panic!("private Core did not connect: {error}"),
            }
        }
    }

    fn wait_exact(child: &mut Child) -> io::Result<ExitStatus> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = child.try_wait()? {
                return Ok(status);
            }
            if Instant::now() >= deadline {
                child.kill()?;
                child.wait()?;
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "private Core child did not exit",
                ));
            }
            std::thread::sleep(Duration::from_millis(30));
        }
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let stop = Self::command(&self.binary, &self.home, &self.socket)
            .args(["server", "stop"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .and_then(|mut child| Self::wait_exact(&mut child));
        let stopped = matches!(&stop, Ok(status) if status.success());
        if !stopped {
            let _ = self.child.kill();
        }
        let served = Self::wait_exact(&mut self.child);
        if !std::thread::panicking() {
            assert!(stopped, "private Core server stop failed: {stop:?}");
            assert!(
                served.is_ok(),
                "private Core serve child was not reaped: {served:?}"
            );
        }
    }
}

fn wait_document(cx: &mut VisualTestContext, version: u64) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let ready = cx.update(|_, app| {
            let observation = app.global_mut::<observation::Observation>();
            observation.poll();
            observation.projection().fresh()
                && observation
                    .projection()
                    .document()
                    .is_some_and(|doc| doc.version == version)
        });
        if ready {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Core document version {version} was not observed"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn wait_edit(cx: &mut VisualTestContext, view: &gpui::Entity<crate::SheprdWindow>) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let done = cx.update(|window, app| {
            view.update(app, |view, app| {
                view.poll_core_workspace_edit(window, app);
                view.core_terminals.workspace_ui.pending.is_none()
            })
        });
        if done {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "Core membership worker did not settle"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn draw_group(cx: &mut VisualTestContext, view: &gpui::Entity<crate::SheprdWindow>) {
    cx.update(|window, app| sidebar::layout_tests::full_draw(window, app).clear(app));
    view.read_with(cx, |view, _| {
        view.sidebar_scroll[0].set_offset(point(px(0.), px(-1_000_000.)));
    });
    for _ in 0..2 {
        cx.update(|window, app| sidebar::layout_tests::full_draw(window, app).clear(app));
    }
}

#[gpui::test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn ui_membership_rebases_disjoint_client_edit_without_touching_terminal(
    cx: &mut gpui::TestAppContext,
) {
    let sandbox = Sandbox::start();
    let first = sandbox.connect();
    let second = sandbox.connect();
    let group = Uuid::from_u128(71);
    let other = Uuid::from_u128(72);
    let core_id = first.hello().core.id.clone();
    let created = edit::apply(
        &first,
        edit::read(&first).unwrap(),
        &Edit::Create {
            id: group,
            label: "One".into(),
        },
    )
    .unwrap();
    assert_eq!(created.version, 1);
    let argv = ["sh".to_owned(), "-c".to_owned(), "sleep 120".to_owned()];
    let terminal = first
        .run(&Run {
            argv: &argv,
            ..Run::default()
        })
        .unwrap();
    let pid = terminal
        .pid
        .expect("private Core reports the terminal process");
    assert!(!terminal.address.contains('/'));
    let backend = CoreBackend::new(sandbox.connect(), 1000);
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|_, app| {
        view.update(app, |view, _| {
            view.core_terminals
                .hub
                .set_backend_for_test(backend.clone())
        });
        if !app.has_global::<observation::Observation>() {
            observation::Observation::install(app);
        }
        app.global_mut::<observation::Observation>().ensure(backend);
    });
    wait_document(cx, 1);
    cx.simulate_resize(size(px(1200.), px(800.)));
    draw_group(cx, &view);
    let select = cx
        .debug_bounds(format!("core-workspace-select-{group}").leak())
        .unwrap();
    cx.simulate_click(select.center(), Modifiers::default());
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.core_terminals
                .workspace_ui
                .selected_group
                .as_ref()
                .map(|key| key.group_id()),
            Some(group)
        );
    });
    cx.update(|_, app| {
        let projection = app.global::<observation::Observation>().projection();
        assert_eq!(projection.core_id(), Some(core_id.as_str()));
        assert!(
            projection
                .terminals()
                .iter()
                .any(|info| info.id.as_str() == terminal.address)
        );
    });
    draw_group(cx, &view);
    let add = cx
        .debug_bounds(format!("core-member-add-{group}-{core_id}-{}", terminal.address).leak())
        .unwrap();

    let second_edit = edit::apply(
        &second,
        created,
        &Edit::Create {
            id: other,
            label: "Two".into(),
        },
    )
    .unwrap();
    assert_eq!(second_edit.version, 2);
    assert!(cx.update(|_, app| {
        app.global::<observation::Observation>()
            .projection()
            .document()
            .is_some_and(|doc| doc.version == 1)
    }));
    cx.simulate_click(add.center(), Modifiers::default());
    let section = cx.debug_bounds("core-workspaces");
    view.read_with(cx, |view, _| {
        assert!(
            view.core_terminals.workspace_ui.pending.is_some(),
            "Add click missed: control={add:?}, section={:?}",
            section
        );
    });
    wait_edit(cx, &view);
    let saved = edit::read(&second).unwrap();
    assert_eq!(saved.version, 3);
    assert_eq!(saved.value.workspaces().len(), 2);
    assert!(
        saved
            .value
            .workspaces()
            .iter()
            .any(|workspace| workspace.id == other)
    );
    assert!(saved.value.workspaces().iter().any(|workspace| {
        workspace.id == group
            && workspace
                .members
                .iter()
                .any(|member| member.core_id == core_id && member.terminal_id == terminal.address)
    }));
    let after_add = second.list(false).unwrap();
    assert_eq!(after_add.terminals.len(), 1);
    assert_eq!(after_add.terminals[0].address, terminal.address);
    assert_eq!(after_add.terminals[0].pid, Some(pid));

    wait_document(cx, 3);
    draw_group(cx, &view);
    let remove = cx
        .debug_bounds(format!("core-member-remove-{group}-{core_id}-{}", terminal.address).leak())
        .unwrap();
    cx.simulate_click(remove.center(), Modifiers::default());
    wait_edit(cx, &view);
    let saved = edit::read(&second).unwrap();
    assert_eq!(saved.version, 4);
    assert!(
        saved
            .value
            .workspaces()
            .iter()
            .find(|workspace| workspace.id == group)
            .unwrap()
            .members
            .is_empty()
    );
    assert!(
        saved
            .value
            .workspaces()
            .iter()
            .any(|workspace| workspace.id == other)
    );
    let listed = second.list(false).unwrap();
    assert_eq!(listed.terminals.len(), 1);
    assert_eq!(listed.terminals[0].address, terminal.address);
    assert_eq!(listed.terminals[0].pid, Some(pid));
}

#[gpui::test]
#[ignore = "needs HERDR_CORE_TEST_BINARY"]
fn ui_refuses_same_group_conflict_and_fences_an_uncertain_version(cx: &mut gpui::TestAppContext) {
    let sandbox = Sandbox::start();
    let first = sandbox.connect();
    let second = sandbox.connect();
    let group = Uuid::from_u128(73);
    let other = Uuid::from_u128(74);
    let core_id = first.hello().core.id.clone();
    let created = edit::apply(
        &first,
        edit::read(&first).unwrap(),
        &Edit::Create {
            id: group,
            label: "One".into(),
        },
    )
    .unwrap();
    assert_eq!(created.version, 1);
    let argv = ["sh".to_owned(), "-c".to_owned(), "sleep 120".to_owned()];
    let terminal = first
        .run(&Run {
            argv: &argv,
            ..Run::default()
        })
        .unwrap();
    let pid = terminal.pid.unwrap();
    let backend = CoreBackend::new(sandbox.connect(), 1000);
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_core_window);
    cx.update(|_, app| {
        view.update(app, |view, _| {
            view.core_terminals
                .hub
                .set_backend_for_test(backend.clone())
        });
        if !app.has_global::<observation::Observation>() {
            observation::Observation::install(app);
        }
        app.global_mut::<observation::Observation>().ensure(backend);
    });
    wait_document(cx, 1);
    cx.simulate_resize(size(px(1200.), px(800.)));
    draw_group(cx, &view);
    let select = cx
        .debug_bounds(format!("core-workspace-select-{group}").leak())
        .unwrap();
    cx.simulate_click(select.center(), Modifiers::default());
    draw_group(cx, &view);
    let add_id = format!("core-member-add-{group}-{core_id}-{}", terminal.address);
    let add = cx.debug_bounds(add_id.clone().leak()).unwrap();

    let renamed = edit::apply(
        &second,
        created,
        &Edit::Rename {
            id: group,
            label: "Changed elsewhere".into(),
        },
    )
    .unwrap();
    assert_eq!(renamed.version, 2);
    assert!(cx.update(|_, app| {
        app.global::<observation::Observation>()
            .projection()
            .document()
            .is_some_and(|doc| doc.version == 1)
    }));
    cx.simulate_click(add.center(), Modifiers::default());
    view.read_with(cx, |view, _| {
        assert!(view.core_terminals.workspace_ui.pending.is_some());
    });
    wait_edit(cx, &view);
    view.read_with(cx, |view, _| {
        let message = &view.flash.as_ref().unwrap().0.text;
        assert!(message.contains(&group.to_string()), "{message}");
    });
    let saved = edit::read(&second).unwrap();
    assert_eq!(saved.version, 2);
    let row = saved
        .value
        .workspaces()
        .iter()
        .find(|row| row.id == group)
        .unwrap();
    assert_eq!(row.label, "Changed elsewhere");
    assert!(row.members.is_empty());
    let listed = second.list(false).unwrap();
    assert_eq!(listed.terminals.len(), 1);
    assert_eq!(listed.terminals[0].address, terminal.address);
    assert_eq!(listed.terminals[0].pid, Some(pid));

    wait_document(cx, 2);
    draw_group(cx, &view);
    let add = cx.debug_bounds(add_id.clone().leak()).unwrap();
    assert!(cx.update(|_, app| {
        let projection = app.global::<observation::Observation>().projection();
        projection.core_id() == Some(core_id.as_str())
            && projection
                .terminals()
                .iter()
                .any(|info| info.id.as_str() == terminal.address)
    }));
    let shared = cx.update(|_, app| EditGuard::begin(app, 2).unwrap());
    let unknown = Err(edit::Error::WriteUnknown {
        version: 2,
        source: Error::Timeout(Method::DocSet),
    });
    EditGuard::finish(&shared, 2, &unknown);
    assert!(cx.update(|_, app| EditGuard::blocks(app, 2)));
    cx.simulate_click(add.center(), Modifiers::default());
    view.read_with(cx, |view, _| {
        assert!(view.core_terminals.workspace_ui.pending.is_none());
        assert!(
            view.flash
                .as_ref()
                .is_some_and(|(flash, _)| flash.text.contains("unconfirmed"))
        );
    });
    assert_eq!(edit::read(&second).unwrap().version, 2);

    let advanced = edit::apply(
        &second,
        saved,
        &Edit::Create {
            id: other,
            label: "Other".into(),
        },
    )
    .unwrap();
    assert_eq!(advanced.version, 3);
    wait_document(cx, 3);
    assert!(!cx.update(|_, app| EditGuard::blocks(app, 3)));
    draw_group(cx, &view);
    let add = cx.debug_bounds(add_id.leak()).unwrap();
    cx.simulate_click(add.center(), Modifiers::default());
    wait_edit(cx, &view);
    let saved = edit::read(&second).unwrap();
    assert_eq!(saved.version, 4);
    assert!(saved.value.workspaces().iter().any(|row| row.id == other));
    assert!(saved.value.workspaces().iter().any(|row| {
        row.id == group
            && row.label == "Changed elsewhere"
            && row
                .members
                .iter()
                .any(|member| member.core_id == core_id && member.terminal_id == terminal.address)
    }));
    let listed = second.list(false).unwrap();
    assert_eq!(listed.terminals.len(), 1);
    assert_eq!(listed.terminals[0].address, terminal.address);
    assert_eq!(listed.terminals[0].pid, Some(pid));
}
