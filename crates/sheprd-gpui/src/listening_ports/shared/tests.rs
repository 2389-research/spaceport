// ABOUTME: Exercises shared scanner leases through private shell producers and real workers.
// ABOUTME: Verifies consumer isolation, exact hosts, retry ownership, and final worker release.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::listening_ports::{
    Daemon, HOST_LIMIT, ListeningPorts, next_scan,
    tests::scan_cost::{private_root, run},
};
use gpui::TestAppContext;
use sheprd_client::ConnectTarget;
use std::{
    fs,
    process::Stdio,
    thread,
    time::{Duration, Instant},
};

fn until(mut done: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(12);
    while !done() {
        assert!(
            Instant::now() < deadline,
            "private scanner condition timed out"
        );
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn leaving_one_consumer_keeps_updates_and_the_final_lease_stops_the_worker() {
    run("listening_ports::shared::tests::lease_fixture", 1);
}

#[gpui::test]
#[ignore = "private scanner lifecycle, executed by its nonignored parent"]
fn lease_fixture(cx: &mut TestAppContext) {
    let root = private_root();
    let daemon = Daemon::of(&ConnectTarget::Socket(root.join("herdr-client.sock"))).unwrap();
    let mut first = ListeningPorts::default();
    let mut second = ListeningPorts::default();
    until(|| {
        cx.update(|cx| {
            first.poll([Host::Local], cx);
            second.poll([Host::Local], cx);
        });
        first.get(&daemon, "w0").is_some() && second.get(&daemon, "w0").is_some()
    });
    let source = first.monitors[&Host::Local].source.as_ref().unwrap();
    assert!(Rc::ptr_eq(
        source,
        second.monitors[&Host::Local].source.as_ref().unwrap()
    ));
    let weak_source = Rc::downgrade(source);
    let worker = Arc::downgrade(&source.state.borrow().worker.as_ref().unwrap().stop);
    let prepared = first.monitors[&Host::Local].reading.clone();
    cx.update(|cx| assert!(first.poll([], cx)));
    assert!(first.get(&daemon, "w0").is_none());
    assert!(worker.upgrade().is_some());
    fs::write(root.join("base"), "3100").unwrap();
    until(|| {
        cx.update(|cx| second.poll([Host::Local], cx));
        second
            .get(&daemon, "w0")
            .is_some_and(|listed| listed.ports[0].number == 3100)
    });
    assert_eq!(prepared.ports.values().next().unwrap()[0].number, 3000);
    assert!(first.get(&daemon, "w0").is_none());
    drop(second);
    assert!(
        weak_source.upgrade().is_none(),
        "the registry keeps only weak leases"
    );
    until(|| worker.upgrade().is_none());
    assert!(
        !prepared.ports.is_empty(),
        "prepared data cannot retain a worker"
    );
    cx.update(|cx| {
        let scans = cx.default_global::<Scans>();
        scans.prune();
        assert!(scans.hosts.is_empty());
    });
}

#[test]
fn each_window_admits_sixteen_exact_hosts_without_a_global_cap() {
    run("listening_ports::shared::tests::host_fixture", 1);
}

#[gpui::test]
#[ignore = "private host shells, executed by their nonignored parent"]
fn host_fixture(cx: &mut TestAppContext) {
    let root = private_root();
    let hosts = |prefix: &str| {
        (0..=HOST_LIMIT)
            .map(|index| Host::Ssh(format!("{prefix}{index}")))
            .collect::<Vec<_>>()
    };
    let left = hosts("left");
    let right = hosts("right");
    let mut first = ListeningPorts::default();
    let mut second = ListeningPorts::default();
    until(|| {
        cx.update(|cx| {
            first.poll(left.clone(), cx);
            second.poll(right.clone(), cx);
        });
        first
            .monitors
            .values()
            .chain(second.monitors.values())
            .all(|monitor| !monitor.reading.ports.is_empty())
    });
    assert_eq!(first.monitors.len(), HOST_LIMIT);
    assert_eq!(second.monitors.len(), HOST_LIMIT);
    assert!(!first.monitors.contains_key(left.last().unwrap()));
    assert!(!second.monitors.contains_key(right.last().unwrap()));
    cx.update(|cx| assert_eq!(cx.default_global::<Scans>().hosts.len(), HOST_LIMIT * 2));
    for host in left.iter().take(HOST_LIMIT) {
        let daemon = Daemon::of(&ConnectTarget::Ssh {
            target: match host {
                Host::Ssh(target) => target.clone(),
                _ => unreachable!(),
            },
            session: "default".into(),
        })
        .unwrap();
        assert!(first.get(&daemon, "w0").is_some());
        assert!(second.get(&daemon, "w0").is_none());
        assert_eq!(first.monitors[host].reading.origin, Origin::new(host));
    }
    let workers = first
        .monitors
        .values()
        .chain(second.monitors.values())
        .map(|monitor| {
            Arc::downgrade(
                &monitor
                    .source
                    .as_ref()
                    .unwrap()
                    .state
                    .borrow()
                    .worker
                    .as_ref()
                    .unwrap()
                    .stop,
            )
        })
        .collect::<Vec<_>>();
    drop((first, second));
    until(|| workers.iter().all(|worker| worker.upgrade().is_none()));
    assert!(
        fs::read_to_string(root.join("scans"))
            .unwrap()
            .lines()
            .count()
            >= HOST_LIMIT * 2
    );
}

#[test]
fn a_broken_shell_keeps_the_last_snapshot_and_reopens_for_the_next_scan() {
    run("listening_ports::shared::tests::reconnect_fixture", 1);
}

#[test]
#[ignore = "private scanner failure, executed by its nonignored parent"]
fn reconnect_fixture() {
    let root = private_root();
    let mut opened = 0;
    let mut shell = None;
    let mut open = || {
        opened += 1;
        let mut command = sheprd_client::new_command("/bin/sh");
        command
            .args([
                "-c",
                r#"printf '%s\n' $$ > "$SPACEPORT_SCAN_FIXTURE_ROOT/shell.pid"; exec /bin/sh -s"#,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        crate::usage::Shell::start(command).map_err(crate::Error::from)
    };
    let mut reading = Reading::default();
    assert!(reading.apply(next_scan(&mut shell, &mut open)));
    let prepared = reading.ports.clone();
    fs::write(root.join("break-shell"), "").unwrap();
    let error = next_scan(&mut shell, &mut open);
    assert!(matches!(
        error,
        Err(crate::Error::ListeningPorts(
            crate::listening_ports::Error::Scan(_)
        ))
    ));
    assert!(shell.is_none());
    assert!(!reading.apply(error));
    assert!(Arc::ptr_eq(&prepared, &reading.ports));
    assert!(reading.error.is_some());
    fs::remove_file(root.join("break-shell")).unwrap();
    fs::write(root.join("base"), "3200").unwrap();
    assert!(reading.apply(next_scan(&mut shell, &mut open)));
    assert!(reading.error.is_none());
    assert_eq!(reading.ports.values().next().unwrap()[0].number, 3200);
    assert_eq!(opened, 2);
}
