// ABOUTME: Exercises the mobile controller against a disposable real SSH and Herdr host.
// ABOUTME: Checks host trust, terminal input, phone key enrollment and session survival.
use spaceport_mobile::{
    EnclaveSigner, MobileRosterRequest, MobileSession, MobileView, SshAuth, SshError, SshHost,
};
#[path = "support/signer.rs"]
mod signer;
use std::{
    env,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

fn setting(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("Set {name} for the disposable SSH fixture."))
}

fn wait(
    session: &MobileSession,
    description: &str,
    matches: impl Fn(&MobileView) -> bool,
) -> MobileView {
    let until = Instant::now() + Duration::from_secs(20);
    let mut version = 0;
    loop {
        if let Some(view) = session.poll(version) {
            version = view.version;
            if matches(&view) {
                return view;
            }
            assert!(!view.disconnected, "{description}: {:?}", view.error);
        }
        assert!(
            Instant::now() < until,
            "Timed out waiting for {description}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

#[test]
#[ignore = "requires an explicitly configured disposable SSH and Herdr fixture"]
fn cold_observer_reads_running_session_without_claiming_its_terminal() {
    let host = SshHost {
        hostname: setting("SPACEPORT_TEST_SSH_HOST"),
        port: setting("SPACEPORT_TEST_SSH_PORT").parse().unwrap(),
        username: setting("SPACEPORT_TEST_SSH_USER"),
        fingerprint: Some(setting("SPACEPORT_TEST_SSH_FINGERPRINT")),
    };
    let login = || SshAuth::Password {
        password: setting("SPACEPORT_TEST_SSH_PASSWORD"),
    };
    let session = setting("SPACEPORT_TEST_HERDR_SESSION");
    let observe = |session: String| MobileRosterRequest::new().read(host.clone(), login(), session);
    assert!(observe(format!("missing-{}", std::process::id())).is_err());
    let active =
        MobileSession::open(host.clone(), login(), session.clone(), 101, 34, 8, 16).unwrap();
    let ready = wait(&active, "writable pane", |view| view.input_target.is_some());
    let before = ready.terminal.as_ref().unwrap();
    let size = (before.width, before.height);
    let passive = observe(session).unwrap();
    assert_eq!(passive.boot_id, ready.boot_id);
    assert_eq!(passive.focused_pane, ready.focused_pane);
    assert!(passive.roster_observed_at_ms.is_some());
    assert!(passive.input_target.is_none() && passive.terminal.is_none());
    let target = ready.input_target.unwrap();
    active
        .send_text(target.clone(), "echo ROSTER-'OBSERVER-OK'".into())
        .unwrap();
    active.send_key(target, "Enter".into(), 0).unwrap();
    let view = wait(
        &active,
        "original terminal still owns focus and size",
        |view| {
            view.terminal.as_ref().is_some_and(|t| {
                t.cells.chunks(usize::from(t.width)).any(|row| {
                    row.iter()
                        .map(|c| c.symbol.as_str())
                        .collect::<String>()
                        .trim()
                        == "ROSTER-OBSERVER-OK"
                })
            })
        },
    );
    let after = view.terminal.unwrap();
    assert_eq!((after.width, after.height), size);
}

#[test]
#[ignore = "requires an explicitly configured disposable SSH and Herdr fixture"]
fn password_then_phone_key_rejoins_the_same_terminal() {
    let mut host = SshHost {
        hostname: setting("SPACEPORT_TEST_SSH_HOST"),
        port: setting("SPACEPORT_TEST_SSH_PORT").parse().unwrap(),
        username: setting("SPACEPORT_TEST_SSH_USER"),
        fingerprint: None,
    };
    let password = setting("SPACEPORT_TEST_SSH_PASSWORD");
    let name = setting("SPACEPORT_TEST_HERDR_SESSION");
    let login = || SshAuth::Password {
        password: password.clone(),
    };
    let open = |host, auth| MobileSession::open(host, auth, name.clone(), 80, 24, 8, 16).unwrap();
    let untrusted = open(host.clone(), login());
    let view = wait(&untrusted, "SSH host key", |v| v.host_challenge.is_some());
    let challenge = view.host_challenge.unwrap();
    assert!(!challenge.changed);
    assert_eq!(
        challenge.fingerprint,
        setting("SPACEPORT_TEST_SSH_FINGERPRINT")
    );
    untrusted.disconnect();
    host.fingerprint = Some(challenge.fingerprint);

    let first = open(host.clone(), login());
    let ready = wait(&first, "writable pane", |v| v.input_target.is_some());
    let target = ready.input_target.unwrap();
    let marker = format!("SPACEPORT_MOBILE_{}_OK", std::process::id());
    let command = format!("echo SPACEPORT_MOBILE_{}'_OK'", std::process::id());
    first.send_paste(target.clone(), command, true).unwrap();
    let has_output = |v: &MobileView| {
        v.terminal.as_ref().is_some_and(|t| {
            t.cells.chunks(usize::from(t.width)).any(|row| {
                row.iter()
                    .map(|cell| cell.symbol.as_str())
                    .collect::<String>()
                    .trim()
                    == marker
            })
        })
    };
    wait(&first, "shell command output", has_output);
    let phone = signer::TestSigner::new();
    first.authorize_phone(phone.point(), phone.clone()).unwrap();
    assert_eq!(phone.count.load(Ordering::Relaxed), 1);
    first.disconnect();

    let second = open(host.clone(), phone.auth());
    let restored = wait(&second, "the original shell output", |v| {
        v.input_target.is_some() && has_output(v)
    });
    let current = restored.input_target.unwrap();
    assert_eq!(current.boot_id, target.boot_id);
    assert_eq!(current.pane_id, target.pane_id);
    assert_ne!(current.connection, target.connection);
    assert!(second.send_text(target, "stale input".into()).is_err());
    // Prepare a second tab and pane using the real Herdr CLI inside the fixture shell.
    let label = format!("mobile-parity-{}", std::process::id());
    second.send_text(current.clone(), format!("herdr tab create --no-focus --label {label} && herdr pane split --current --direction right")).unwrap();
    second.send_key(current.clone(), "Enter".into(), 0).unwrap();
    let split = wait(&second, "second tab and split pane", |v| {
        v.tabs.iter().any(|t| t.label == label)
            && v.panes
                .iter()
                .filter(|p| Some(&p.tab_id) == v.focused_tab.as_ref())
                .count()
                > 1
            && v.input_target.is_some()
    });
    let original_tab = split.focused_tab.unwrap();
    let map = split
        .pane_map
        .expect("a right split must supply visible pane rectangles");
    assert!(map.panes.iter().any(|left| {
        map.panes
            .iter()
            .any(|right| left.x + left.width <= right.x && left.y == right.y)
    }));
    let other_pane = map
        .panes
        .iter()
        .find(|p| Some(&p.id) != split.focused_pane.as_ref())
        .unwrap()
        .id
        .clone();
    let other_tab = split
        .tabs
        .iter()
        .find(|t| t.label == label)
        .unwrap()
        .id
        .clone();
    let third = open(host, phone.auth());
    let third_view = wait(&third, "second client", |v| v.input_target.is_some());
    third
        .focus_tab(third_view.boot_id.clone(), other_tab.clone())
        .unwrap();
    wait(&third, "independent tab selection", |v| {
        v.focused_tab.as_ref() == Some(&other_tab) && v.input_target.is_some()
    });
    let unchanged = second.poll(0).unwrap();
    assert_eq!(unchanged.focused_tab.as_ref(), Some(&original_tab));
    third
        .focus_tab(third_view.boot_id, original_tab.clone())
        .unwrap();
    wait(&third, "return to the shared tab", |v| {
        v.focused_tab.as_ref() == Some(&original_tab) && v.input_target.is_some()
    });
    let target_boot = current.boot_id.clone();
    second
        .focus_pane(current.boot_id, other_pane.clone())
        .unwrap();
    wait(&third, "shared focused pane", |v| {
        v.focused_pane.as_ref() == Some(&other_pane) && v.input_target.is_some()
    });
    // Simulate a narrow phone claiming the shared tab after the laptop goes idle.
    std::thread::sleep(spaceport_client::projection::REASSERT_AFTER_INPUT);
    third.resize(target_boot, 48, 18, 8, 16).unwrap();
    let phone_view = wait(&third, "phone viewport", |v| v.input_target.is_some());
    let terminal = phone_view.terminal.unwrap();
    let expected = format!("MOB {} {}", terminal.height, terminal.width);
    let phone_target = phone_view.input_target.unwrap();
    third
        .send_text(phone_target.clone(), "echo MOB $(stty size)".into())
        .unwrap();
    third.send_key(phone_target, "Enter".into(), 0).unwrap();
    let has_size = |v: &MobileView| {
        v.terminal.as_ref().is_some_and(|t| {
            t.cells.chunks(usize::from(t.width)).any(|row| {
                row.iter()
                    .map(|c| c.symbol.as_str())
                    .collect::<String>()
                    .trim()
                    == expected
            })
        })
    };
    wait(&third, "shell reports the smaller PTY size", has_size);
    let laptop_result = wait(&second, "other client sees the new PTY size", has_size);
    // Herdr 0.9.5 gives the clients different viewport projections of this same PTY.
    let laptop = laptop_result.terminal.unwrap();
    assert!(laptop.width > terminal.width && laptop.height > terminal.height);
    third.disconnect();
    second.disconnect();
}

#[test]
#[ignore = "requires an explicitly configured disposable SSH and Herdr fixture"]
fn direct_attach_round_trips_shell_bytes_and_leaves_the_same_pane_alive() {
    let host = SshHost {
        hostname: setting("SPACEPORT_TEST_SSH_HOST"),
        port: setting("SPACEPORT_TEST_SSH_PORT").parse().unwrap(),
        username: setting("SPACEPORT_TEST_SSH_USER"),
        fingerprint: Some(setting("SPACEPORT_TEST_SSH_FINGERPRINT")),
    };
    let session = MobileSession::open(
        host,
        SshAuth::Password {
            password: setting("SPACEPORT_TEST_SSH_PASSWORD"),
        },
        setting("SPACEPORT_TEST_HERDR_SESSION"),
        80,
        24,
        8,
        16,
    )
    .unwrap();
    let view = wait(&session, "initial terminal", |view| {
        view.input_target.is_some()
    });
    let target = view.input_target.unwrap();
    session.poll_for_attach(0).unwrap();
    let pty = session.attach_terminal(target.clone(), 80, 24).unwrap();
    let until = Instant::now() + Duration::from_secs(8);
    while pty.read_waiting().unwrap().is_empty() {
        assert!(Instant::now() < until, "the attached terminal must paint");
    }
    pty.write(
        b"printf '\\123\\120\\101\\103\\105\\120\\117\\122\\124\\055\\120\\124\\131\\n'\r".to_vec(),
    )
    .unwrap();
    let mut received = Vec::new();
    while !received
        .windows(b"SPACEPORT-PTY".len())
        .any(|run| run == b"SPACEPORT-PTY")
    {
        assert!(
            Instant::now() < until,
            "the selected shell must execute the command"
        );
        received.extend(pty.read_waiting().unwrap());
        assert!(received.len() <= 4 * 1024 * 1024);
    }
    pty.close();
    assert_eq!(
        session.poll_for_attach(0).unwrap().input_target,
        Some(target)
    );
    session.disconnect();
}

#[test]
#[ignore = "requires an explicitly configured disposable SSH and Herdr fixture"]
fn file_delivery_confirms_bytes_and_rejects_the_previous_destination() {
    use spaceport_mobile::{MobileError, MobileFileTransfer};
    let session = MobileSession::open(
        SshHost {
            hostname: setting("SPACEPORT_TEST_SSH_HOST"),
            port: setting("SPACEPORT_TEST_SSH_PORT").parse().unwrap(),
            username: setting("SPACEPORT_TEST_SSH_USER"),
            fingerprint: Some(setting("SPACEPORT_TEST_SSH_FINGERPRINT")),
        },
        SshAuth::Password {
            password: setting("SPACEPORT_TEST_SSH_PASSWORD"),
        },
        setting("SPACEPORT_TEST_HERDR_SESSION"),
        110,
        28,
        8,
        16,
    )
    .unwrap();
    let ready = wait(&session, "file destination", |view| {
        view.input_target.is_some()
    });
    let original = ready.input_target.unwrap();
    let transfer = MobileFileTransfer::new();
    let bytes = b"SPACEPORT-VERIFIED-FILE".to_vec();
    let file = session
        .stage_draft_file(
            original.clone(),
            "agent's $notes.txt".into(),
            bytes.clone(),
            transfer.clone(),
        )
        .unwrap();
    assert_eq!(transfer.bytes_sent(), bytes.len() as u64);
    let path = file.paste_path();
    session
        .send_paste(
            original.clone(),
            format!("cat -- {path}; echo; rm -- {path}"),
            true,
        )
        .unwrap();
    file.keep();
    wait(&session, "verified file content", |view| {
        view.terminal.as_ref().is_some_and(|screen| {
            screen.cells.chunks(usize::from(screen.width)).any(|row| {
                row.iter()
                    .map(|cell| cell.symbol.as_str())
                    .collect::<String>()
                    .trim()
                    == "SPACEPORT-VERIFIED-FILE"
            })
        })
    });
    session
        .create_tab(
            ready.boot_id,
            ready.focused_workspace.unwrap(),
            ready.focused_tab.unwrap(),
        )
        .unwrap();
    wait(&session, "different terminal", |v| {
        v.input_target
            .as_ref()
            .is_some_and(|t| t.pane_id != original.pane_id)
    });
    assert!(matches!(
        session.send_paste(original.clone(), path, false),
        Err(MobileError::StaleView)
    ));
    assert!(matches!(
        session.stage_draft_file(original, "stale".into(), bytes, MobileFileTransfer::new()),
        Err(MobileError::StaleView)
    ));
    session.disconnect();
}
#[path = "support/pane_actions.rs"]
mod pane_actions;
fn page(port: u16) -> String {
    use std::{
        io::{Read, Write},
        net::{Ipv4Addr, TcpStream},
    };
    let mut connection = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
    connection
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    connection
        .write_all(b"GET / HTTP/1.0\r\nHost: localhost\r\n\r\n")
        .unwrap();
    let mut text = String::new();
    connection.read_to_string(&mut text).unwrap();
    text
}

#[test]
#[ignore = "requires a disposable SSH host with the Spaceport loopback trial HTTP fixture"]
fn chosen_service_survives_park_and_closes_on_stop() {
    let setting = |name| env::var(name).unwrap();
    let host = SshHost {
        hostname: setting("SPACEPORT_TEST_SSH_HOST"),
        port: setting("SPACEPORT_TEST_SSH_PORT").parse().unwrap(),
        username: setting("SPACEPORT_TEST_SSH_USER"),
        fingerprint: Some(setting("SPACEPORT_TEST_SSH_FINGERPRINT")),
    };
    let session = MobileSession::open(
        host,
        SshAuth::Password {
            password: setting("SPACEPORT_TEST_SSH_PASSWORD"),
        },
        setting("SPACEPORT_TEST_HERDR_SESSION"),
        80,
        24,
        8,
        16,
    )
    .unwrap();
    let target = setting("SPACEPORT_TEST_SERVICE_PORT").parse().unwrap();
    wait(&session, "attached pane", |view| {
        view.input_target.is_some()
    });
    assert!(session.start_service_preview(0).is_err());
    let port = session.start_service_preview(target).unwrap();
    assert!(session.start_service_preview(target).is_err());
    assert!(page(port).contains("Spaceport loopback trial"));
    session.park();
    assert!(page(port).contains("Spaceport loopback trial"));
    use std::io::{Read, Write};
    use std::net::{Ipv4Addr, TcpStream};
    let mut unfinished = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
    unfinished
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    unfinished.write_all(b"GET /partial HTTP/1.1\r\n").unwrap();
    session.stop_service_preview().unwrap();
    let mut byte = [0];
    match unfinished.read(&mut byte) {
        Ok(0) => {}
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted
            ) => {}
        result => panic!("the existing browser connection survived stop: {result:?}"),
    }
    assert!(std::net::TcpStream::connect((std::net::Ipv4Addr::LOCALHOST, port)).is_err());
    session.disconnect();
}
