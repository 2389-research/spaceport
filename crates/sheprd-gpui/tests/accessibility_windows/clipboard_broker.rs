// ABOUTME: Qualifies clipboard admission and ownership using only authored Windows data.
// ABOUTME: Requires a disposable empty clipboard and preserves any foreign replacement.
use super::{CAPTURE_LIMIT, Duration, GuiTarget, Instant, Read, Running, fs, thread};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    process::ExitStatus,
};

fn finished(running: &mut Running) -> Option<ExitStatus> {
    running
        .observer
        .as_mut()
        .unwrap()
        .try_wait()
        .unwrap()
        .map(|_| running.observer.take().unwrap().wait().unwrap())
}

fn await_exit(running: &mut Running, deadline: Instant) -> Option<ExitStatus> {
    loop {
        if let Some(status) = finished(running) {
            return Some(status);
        }
        if Instant::now() >= deadline {
            return None;
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn preserve_owner(running: Running) {
    eprintln!(
        "Clipboard cleanup is unconfirmed; retaining owner PID {} and private root {} until its guardian exits",
        running.observer.as_ref().unwrap().id(),
        running.sandbox.dir.display()
    );
    // If the cleanup reaper cannot start, dropping this wrapper must not kill the owner.
    let retained = std::mem::ManuallyDrop::new(running);
    let reaper = thread::Builder::new()
        .name("clipboard-cleanup".into())
        .spawn(move || {
            let mut running = std::mem::ManuallyDrop::into_inner(retained);
            let mut reported = false;
            loop {
                match running.observer.as_mut().unwrap().try_wait() {
                    Ok(Some(_)) => {
                        if let Some(mut child) = running.observer.take()
                            && let Err(error) = child.wait()
                        {
                            eprintln!("Terminal clipboard owner wait failed: {error}");
                        }
                        // Only a terminal owned handle permits removal of the private root.
                        return;
                    }
                    Ok(None) => {},
                    Err(error) if !reported => {
                        reported = true;
                        eprintln!("Clipboard cleanup reaper cannot inspect its owner; retaining it: {error}");
                    }
                    Err(_) => {},
                }
                thread::sleep(Duration::from_millis(500));
            }
        });
    if let Err(error) = reaper {
        eprintln!("Clipboard owner and private root remain retained without a reaper: {error}");
    }
}

fn owner_check(parent_failure: bool) -> String {
    run_owner(parent_failure, None)
}

pub(super) fn application_check(target: GuiTarget, scenario: &str) -> String {
    run_owner(false, Some((target, scenario)))
}

fn run_owner(parent_failure: bool, application: Option<(GuiTarget, &str)>) -> String {
    let mut running = Running::prepared();
    for (name, source) in [
        (
            "native_clipboard.cs",
            include_str!("../support/native_clipboard.cs"),
        ),
        (
            "native_clipboard.ps1",
            include_str!("../support/native_clipboard.ps1"),
        ),
        (
            "uia_clipboard.ps1",
            include_str!("../support/uia_clipboard.ps1"),
        ),
    ] {
        fs::write(running.sandbox.dir.join(name), source).unwrap();
    }
    let script = if let Some((target, _)) = &application {
        fs::write(
            running
                .sandbox
                .dir
                .join("config/spaceport/config.local.toml"),
            "language = \"en-US\"\n",
        )
        .unwrap();
        running.start_gui(*target);
        running.wait_tree("native paste input", |tree| {
            tree.contains(if matches!(target, GuiTarget::ChatClipboard) {
                "Edit 'Message the agent'"
            } else {
                "Edit 'Native standalone field'"
            })
        });
        "uia_clipboard.ps1"
    } else {
        "native_clipboard.ps1"
    };
    let mut command = running.powershell(&running.sandbox.dir.join(script), "clipboard-broker.log");
    command.args([
        "-Scenario",
        if let Some((_, scenario)) = application {
            scenario
        } else if parent_failure {
            "Watchdog"
        } else {
            "Ownership"
        },
    ]);
    running.observer = Some(command.spawn().unwrap());
    let stop = running.sandbox.dir.join("clipboard-stop");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let startup = Instant::now() + Duration::from_secs(60);
        while !stop.with_extension("started").exists() {
            if let Some(status) = finished(&mut running) {
                return status;
            }
            assert!(
                Instant::now() < startup,
                "native clipboard guardian did not start"
            );
            thread::sleep(Duration::from_millis(25));
        }
        // Start the publication budget only after compilation and guardian startup.
        let deadline =
            Instant::now() + Duration::from_secs(if application.is_some() { 90 } else { 35 });
        if parent_failure {
            while !stop.with_extension("ready").exists() {
                assert!(
                    running
                        .observer
                        .as_mut()
                        .unwrap()
                        .try_wait()
                        .unwrap()
                        .is_none(),
                    "native clipboard owner exited before publication"
                );
                assert!(
                    Instant::now() < deadline,
                    "native clipboard owner did not publish its authored payload"
                );
                thread::sleep(Duration::from_millis(25));
            }
            panic!("expected parent failure after authored clipboard publication");
        }
        await_exit(&mut running, deadline)
            .expect("native clipboard owner did not exit within its publication budget")
    }));
    let (status, failure) = match result {
        Ok(status) => (status, None),
        Err(failure) => {
            let requested = fs::write(&stop, "stop");
            // Retain Running and its owner until cooperative cleanup or the independent guardian exits.
            // The stop file triggers the guardian immediately; its clipboard-open deadline is two seconds.
            let status = match await_exit(&mut running, Instant::now() + Duration::from_secs(35)) {
                Some(status) => status,
                None => {
                    if stop.with_extension("cleaned").exists() {
                        // Cleanup is proved and the cancelled owner can never publish again.
                        super::stop_children([&mut running.observer]);
                    } else {
                        preserve_owner(running);
                    }
                    std::panic::resume_unwind(failure);
                }
            };
            if let Err(error) = requested {
                eprintln!(
                    "Clipboard stop request failed; independent guardian handled exit: {error}"
                );
            }
            (status, Some(failure))
        }
    };
    let mut bytes = Vec::new();
    fs::File::open(running.sandbox.dir.join("clipboard-broker.log"))
        .unwrap()
        .take((CAPTURE_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .unwrap();
    assert!(
        bytes.len() <= CAPTURE_LIMIT,
        "native clipboard receipts exceed bound"
    );
    let output = String::from_utf8(bytes).unwrap();
    eprintln!("{output}");
    if parent_failure {
        let failure =
            failure.expect("the native parent-failure check must reach its deliberate panic");
        assert_eq!(
            failure.downcast_ref::<&str>(),
            Some(&"expected parent failure after authored clipboard publication")
        );
        assert_eq!(status.code(), Some(3), "{output}");
    } else {
        if let Some(failure) = failure {
            std::panic::resume_unwind(failure);
        }
        assert!(status.success(), "{status}: {output}");
    }
    assert!(
        stop.with_extension("cleaned").exists(),
        "native clipboard owner exited without its cleanup receipt: {output}"
    );
    assert!(
        output.contains("CLIPBOARD_GUARDIAN cleanup_safe=true"),
        "{output}"
    );
    output
}

#[test]
#[ignore = "requires a disposable Windows desktop with an initially empty clipboard; refuses nonempty data unchanged"]
fn native_clipboard_fixture_preserves_foreign_content_and_cleans_its_own() {
    let output = owner_check(false);
    for receipt in [
        "CLIPBOARD_GUARD nonempty_refused=true publication_preserved=true",
        "CLIPBOARD_BROKER authored_utf8_bytes=262144",
        "CLIPBOARD_BROKER authored_utf8_bytes=2097153",
        "CLIPBOARD_GUARD foreign_publication_preserved=true",
        "CLIPBOARD_GUARD same_hwnd_stale_sequence_refused=true",
        "CLIPBOARD_GUARD bounded_hold_released=true callback_failure_released=true",
        "CLIPBOARD_GUARD failure_cleanup=true empty=true",
        "CLIPBOARD BROKER native PASS:",
    ] {
        assert!(output.contains(receipt), "{output}");
    }
}

#[test]
#[ignore = "requires a disposable Windows desktop with an initially empty clipboard; preserves the parent failure until guardian cleanup"]
fn native_parent_failure_keeps_clipboard_owner_alive_until_guardian_cleanup() {
    let output = owner_check(true);
    assert!(
        output.contains("CLIPBOARD_GUARDIAN publication_ready=true"),
        "{output}"
    );
    // A fresh native empty-only admission proves the guardian actually removed its data.
    owner_check(false);
}
