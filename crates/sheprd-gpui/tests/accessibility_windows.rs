// ABOUTME: Reads a running window through native Windows UI Automation.
// ABOUTME: Checks keyboard focus with an isolated app and pinned daemon.
//! Requires an active Windows desktop and an explicitly selected daemon
//! executable (`HERDR_TEST_BINARY`), like the live GUI test.
#![cfg(windows)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::disallowed_methods)]
#[path = "../../test-support/sandbox.rs"]
mod sandbox;

use sandbox::{Sandbox, daemon_binary, stop_children};
use std::{
    fs,
    io::{self, Read},
    path::Path,
    process::Child,
    thread,
    time::{Duration, Instant},
};

struct Running {
    command: Option<Child>,
    daemon: Option<Child>,
    gui: Option<Child>,
    sandbox: Sandbox,
}

impl Drop for Running {
    fn drop(&mut self) {
        stop_children([&mut self.command, &mut self.gui, &mut self.daemon]);
    }
}

impl Running {
    fn empty() -> Self {
        let sandbox = Sandbox::new();
        fs::write(
            sandbox.dir.join("powershell_modules.ps1"),
            include_str!("support/powershell_modules.ps1"),
        )
        .unwrap();
        Self {
            command: None,
            daemon: None,
            gui: None,
            sandbox,
        }
    }

    fn start() -> Self {
        let mut running = Self::empty();
        running.check_shell();
        let config = running.sandbox.dir.join("config/spaceport");
        fs::create_dir_all(&config).unwrap();
        fs::write(
            config.join("view.toml"),
            "language = \"en-US\"\n[ui]\nprompt_new_workspace_name = true\n",
        )
        .unwrap();
        running.daemon = Some(
            running
                .sandbox
                .command(daemon_binary(), "daemon.log")
                .arg("server")
                .spawn()
                .unwrap(),
        );
        running
            .sandbox
            .wait_for_daemon(running.daemon.as_mut().unwrap(), Duration::from_secs(20));
        running.gui = Some(
            running
                .sandbox
                .command(env!("CARGO_BIN_EXE_spaceport"), "gui.log")
                .arg("--socket")
                .arg(running.sandbox.socket())
                .spawn()
                .unwrap(),
        );
        fs::write(
            running.sandbox.dir.join("uia_tree.ps1"),
            include_str!("support/uia_tree.ps1"),
        )
        .unwrap();
        running
    }

    fn check_shell(&mut self) {
        let started = Instant::now();
        let output = self
            .shell_probe(
                "[Console]::WriteLine('PRIVATE POWERSHELL READY')",
                Duration::from_secs(60),
            )
            .expect("the private PowerShell host must execute a plain script before UI Automation");
        assert!(output.contains("PRIVATE POWERSHELL READY"), "{output}");
        eprintln!(
            "PRIVATE POWERSHELL READY: cleared environment and private capture in {} ms",
            started.elapsed().as_millis()
        );
    }

    fn shell_probe(&mut self, body: &str, limit: Duration) -> io::Result<String> {
        let script = self.sandbox.dir.join("shell-probe.ps1");
        fs::write(
            &script,
            format!("param([int]$ProcessId, [string]$Keys)\n{body}\n"),
        )?;
        self.script_output(&script, "-Keys", "", limit)
    }

    fn script_output(
        &mut self,
        script: &Path,
        flag: &str,
        value: &str,
        limit: Duration,
    ) -> io::Result<String> {
        if let Some(gui) = &mut self.gui
            && gui.try_wait()?.is_some()
        {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "the owned GUI exited",
            ));
        }
        let process = self.gui.as_ref().map_or(0, Child::id);
        let capture = self.sandbox.dir.join("uia-capture.log");
        let mut command = self.sandbox.command("powershell.exe", "uia-capture.log");
        // Windows PowerShell uses these system paths while loading cmdlets.
        for name in ["ProgramFiles", "ProgramFiles(x86)"] {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        self.command = Some(
            command
                .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
                .arg(script)
                .args(["-ProcessId", &process.to_string(), flag, value])
                .spawn()?,
        );
        let result = (|| {
            let deadline = Instant::now() + limit;
            loop {
                if let Some(status) = self.command.as_mut().unwrap().try_wait()? {
                    let mut bytes = Vec::new();
                    fs::File::open(&capture)?
                        .take(1_048_577)
                        .read_to_end(&mut bytes)?;
                    if bytes.len() > 1_048_576 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "UI Automation capture exceeds 1 MiB",
                        ));
                    }
                    if !status.success() {
                        return Err(io::Error::other(format!(
                            "UI Automation failed: {status}: {}",
                            String::from_utf8_lossy(&bytes)
                        )));
                    }
                    return String::from_utf8(bytes)
                        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
                }
                if Instant::now() >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "UI Automation command exceeded its deadline",
                    ));
                }
                thread::sleep(Duration::from_millis(10));
            }
        })();
        stop_children([&mut self.command]);
        if self.command.as_mut().unwrap().try_wait()?.is_none() {
            return Err(io::Error::other("UI Automation child was not reaped"));
        }
        result.map_err(|error| {
            if error.kind() != io::ErrorKind::TimedOut {
                return error;
            }
            let mut bytes = Vec::new();
            let capture = fs::File::open(&capture)
                .and_then(|file| file.take(1_048_576).read_to_end(&mut bytes));
            io::Error::new(
                error.kind(),
                format!(
                    "{error}; private capture read: {capture:?}\n{}",
                    String::from_utf8_lossy(&bytes)
                ),
            )
        })
    }

    fn read(&mut self, keys: &str) -> String {
        self.script_output(
            &self.sandbox.dir.join("uia_tree.ps1"),
            "-Keys",
            keys,
            Duration::from_secs(20),
        )
        .unwrap()
    }

    fn invoke(&mut self, name: &str) {
        self.script_output(
            &self.sandbox.dir.join("uia_tree.ps1"),
            "-Invoke",
            name,
            Duration::from_secs(20),
        )
        .unwrap();
    }

    fn wait_tree(&mut self, label: &str, ready: impl Fn(&str) -> bool) -> String {
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            let tree = self.read("");
            if ready(&tree) {
                return tree;
            }
            assert!(Instant::now() < deadline, "{label} did not appear:\n{tree}");
            thread::sleep(Duration::from_millis(200));
        }
    }
}

#[test]
fn a_private_powershell_host_executes_a_plain_script() {
    Running::empty().check_shell();
}

#[test]
fn a_private_powershell_host_sets_utf8_output() {
    let output = Running::empty()
        .shell_probe(
            "[Console]::WriteLine('ENCODING START')\n\
             [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)\n\
             [Console]::WriteLine('ENCODING READY ' + [char]0x2603)",
            Duration::from_secs(20),
        )
        .unwrap();
    assert!(output.contains("ENCODING READY \u{2603}"), "{output}");
}

#[test]
fn a_private_powershell_host_executes_a_cmdlet() {
    let output = Running::empty()
        .shell_probe(
            "[Console]::WriteLine('CMDLET START')\n\
             . \"$PSScriptRoot\\powershell_modules.ps1\"\n\
             Write-Output 'CMDLET READY'",
            Duration::from_secs(20),
        )
        .unwrap();
    assert!(
        output.contains("POWERSHELL stage: system modules ready"),
        "{output}"
    );
    assert!(output.contains("CMDLET READY"), "{output}");
}

#[test]
fn a_command_timeout_reaps_its_owned_child() {
    let mut running = Running::empty();
    let script = running.sandbox.dir.join("sleep.ps1");
    fs::write(
        &script,
        "param([int]$ProcessId, [string]$Keys)\nwhile ($true) { Start-Sleep -Seconds 30 }\n",
    )
    .unwrap();
    let error = running
        .script_output(&script, "-Keys", "", Duration::from_millis(100))
        .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert!(
        running
            .command
            .as_mut()
            .unwrap()
            .try_wait()
            .unwrap()
            .is_some()
    );
}

#[path = "accessibility_windows/workspace_close.rs"]
mod workspace_close;

#[test]
#[ignore = "requires an active Windows desktop and HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn ui_automation_names_the_chrome_and_prefix_region_commands_reach_it() {
    let mut running = Running::start();

    // AccessKit builds the tree once a client first asks, a frame later, and
    // the sidebar arrives with the first snapshot after that.
    let expected = [
        "MenuBar 'Menu bar'",
        "MenuItem 'File' [Collapsed]",
        "Group 'Sidebar'",
        "List 'Workspaces'",
        "ListItem '",
        "Button 'New workspace' [Invoke]",
        "List 'Agents'",
        "Tab 'Activity'",
        "Button 'Settings' [Invoke]",
        "Button 'Devices: ",
        "TabItem '1' [selected=True]",
        "Button 'New tab' [Invoke]",
        "Document 'Terminal'",
        "StatusBar 'Status bar'",
        "Button 'Theme' [Invoke]",
        "Text 'Version ",
    ];
    let tree = running.wait_tree("main chrome", |tree| {
        expected.iter().all(|name| tree.contains(name))
    });
    eprintln!("{tree}");
    let version = tree
        .lines()
        .find(|line| line.trim_start().starts_with("Text 'Version "))
        .unwrap();
    assert!(
        !version.contains("Invoke") && !version.contains("Value"),
        "the version label has no action: {version}"
    );
    // Back and Forward have no history to go to yet.
    assert!(tree.contains("Button 'Back' [disabled]"), "{tree}");

    let focus = running.read("^b{F6}|{TAB}|^b{F6}|^b{F6}|+{TAB}|{ESC}|^b+{F6}");
    eprintln!("{focus}");
    let moves: Vec<&str> = focus
        .lines()
        .filter_map(|line| line.split_once(" -> ").map(|(_, focused)| focused))
        .collect();
    assert_eq!(moves.len(), 7, "{focus}");
    // Title bar, then Tab skips the unavailable Back and Forward.
    assert!(moves[0].starts_with("Button 'Sidebar'"), "{focus}");
    assert!(
        !moves[1].contains("'Back'") && moves[1] != moves[0],
        "{focus}"
    );
    // The sidebar starts at its activity bar.
    assert!(moves[2].starts_with("TabItem 'Workspaces"), "{focus}");
    // The status bar, and Shift-Tab wraps within it.
    assert!(
        moves[3].starts_with("Button ") || moves[3].starts_with("Text "),
        "{focus}"
    );
    assert_ne!(moves[4], moves[3], "{focus}");
    // Escape and the terminal; Prefix, Shift-F6 goes back to the last region.
    assert!(moves[5].starts_with("Document 'Terminal'"), "{focus}");
    assert!(!moves[6].starts_with("Document"), "{focus}");
}
