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

const CAPTURE_LIMIT: usize = 1_048_576;

struct Running {
    command: Option<Child>,
    daemon: Option<Child>,
    gui: Option<Child>,
    observer: Option<Child>,
    sandbox: Sandbox,
}

#[derive(Clone, Copy)]
enum GuiTarget {
    // keep-sorted start
    ChatAccessibility,
    ChatClipboard,
    ClipboardField,
    Core(core::Target),
    Images(uuid::Uuid),
    Local,
    SettingsKeyboard,
    SettingsScaling,
    ShortcutFenceObserver,
    ShortcutObserver,
    Socket,
    // keep-sorted end
}

impl Drop for Running {
    fn drop(&mut self) {
        stop_children([
            &mut self.command,
            &mut self.observer,
            &mut self.gui,
            &mut self.daemon,
        ]);
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
            observer: None,
            sandbox,
        }
    }

    fn start() -> Self {
        let mut running = Self::prepared();
        running.start_daemon();
        running.start_gui(GuiTarget::Socket);
        running
    }

    fn prepared() -> Self {
        let mut running = Self::empty();
        running.check_shell();
        let config = running.sandbox.dir.join("config/spaceport");
        fs::create_dir_all(&config).unwrap();
        fs::write(
            config.join("view.toml"),
            "language = \"en-US\"\n[ui]\nprompt_new_workspace_name = true\n",
        )
        .unwrap();
        fs::write(
            running.sandbox.dir.join("uia_tree.ps1"),
            include_str!("support/uia_tree.ps1"),
        )
        .unwrap();
        running
    }

    fn start_daemon(&mut self) {
        assert!(self.daemon.is_none());
        self.daemon = Some(
            self.sandbox
                .command(daemon_binary(), "daemon.log")
                .arg("server")
                .spawn()
                .unwrap(),
        );
        self.sandbox
            .wait_for_daemon(self.daemon.as_mut().unwrap(), Duration::from_secs(20));
    }

    fn start_gui(&mut self, target: GuiTarget) {
        assert!(self.gui.is_none());
        let mut command = self
            .sandbox
            .command(env!("CARGO_BIN_EXE_spaceport"), "gui.log");
        match target {
            GuiTarget::Core(target) => {
                command.arg("--socket").arg(self.sandbox.socket());
                target.apply(&mut command);
            }
            GuiTarget::ChatClipboard | GuiTarget::ClipboardField => {
                command
                    .arg("--sidebar-test")
                    .env(
                        "HERDR_TEST_RICH_CHAT",
                        if matches!(target, GuiTarget::ChatClipboard) {
                            "clipboard"
                        } else {
                            "clipboard-field"
                        },
                    )
                    .env("HERDR_TEST_RICH_CHAT_HOLD_MS", "180000");
            }
            GuiTarget::ChatAccessibility => {
                command
                    .arg("--sidebar-test")
                    .env("HERDR_TEST_RICH_CHAT", "accessibility")
                    .env("HERDR_TEST_RICH_CHAT_HOLD_MS", "180000")
                    .env(
                        "HERDR_TEST_RICH_CHAT_CAPTURE",
                        self.sandbox.dir.join("native-chat.png"),
                    );
            }
            GuiTarget::Socket
            | GuiTarget::Images(_)
            | GuiTarget::ShortcutObserver
            | GuiTarget::ShortcutFenceObserver
            | GuiTarget::SettingsKeyboard
            | GuiTarget::SettingsScaling => {
                command.arg("--socket").arg(self.sandbox.socket());
                if let GuiTarget::Images(token) = target {
                    command.env("HERDR_TEST_NATIVE_IMAGES", token.to_string());
                }
                if matches!(target, GuiTarget::SettingsKeyboard) {
                    command.env("HERDR_TEST_SETTINGS_KEYBOARD", "1");
                }
                if matches!(target, GuiTarget::SettingsScaling) {
                    command.env("HERDR_TEST_SETTINGS_KEYBOARD", "scale");
                }
                if matches!(
                    target,
                    GuiTarget::ShortcutObserver | GuiTarget::ShortcutFenceObserver
                ) {
                    command.env("HERDR_TEST_SHORTCUT_OBSERVER", "1");
                }
                if matches!(target, GuiTarget::ShortcutFenceObserver) {
                    command.env("HERDR_TEST_SHORTCUT_FENCES", "1");
                }
            }
            GuiTarget::Local => {
                // Refuse an installed system candidate before testing the missing-runtime card.
                let path = command
                    .get_envs()
                    .find(|(key, _)| *key == "PATH")
                    .and_then(|(_, value)| value)
                    .unwrap();
                for directory in std::env::split_paths(path) {
                    assert!(!directory.join("herdr.exe").exists());
                }
                for directory in [".local/bin", ".cargo/bin"] {
                    assert!(!self.sandbox.dir.join(directory).join("herdr.exe").exists());
                }
            }
        }
        self.gui = Some(command.spawn().unwrap());
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
        self.script_output(&script, "-Keys", "", None, limit)
    }

    fn script_output(
        &mut self,
        script: &Path,
        flag: &str,
        value: &str,
        expected_button: Option<&str>,
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
        let capture = self.sandbox.dir.join("uia-capture.log");
        let mut command = self.powershell(script, "uia-capture.log");
        command.args([flag, value]);
        if let Some(button) = expected_button {
            command.args(["-ExpectedButton", button]);
        }
        self.command_output(command, &capture, limit)
    }

    fn powershell(&self, script: &Path, log: &str) -> std::process::Command {
        let process = self.gui.as_ref().map_or(0, Child::id);
        let mut command = self.sandbox.command("powershell.exe", log);
        // Windows PowerShell uses these system paths while loading cmdlets.
        for name in ["ProgramFiles", "ProgramFiles(x86)"] {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        command
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
            .arg(script)
            .args(["-ProcessId", &process.to_string()]);
        command
    }

    fn command_output(
        &mut self,
        mut command: std::process::Command,
        capture: &Path,
        limit: Duration,
    ) -> io::Result<String> {
        self.command = Some(command.spawn()?);
        let result = (|| {
            let deadline = Instant::now() + limit;
            loop {
                if let Some(status) = self.command.as_mut().unwrap().try_wait()? {
                    let mut bytes = Vec::new();
                    fs::File::open(capture)?
                        .take((CAPTURE_LIMIT + 1) as u64)
                        .read_to_end(&mut bytes)?;
                    if bytes.len() > CAPTURE_LIMIT {
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
            let capture = fs::File::open(capture)
                .and_then(|file| file.take(CAPTURE_LIMIT as u64).read_to_end(&mut bytes));
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
            None,
            Duration::from_secs(20),
        )
        .unwrap()
    }

    fn press_button(&mut self, keys: &str, button: &str) -> io::Result<String> {
        self.script_output(
            &self.sandbox.dir.join("uia_tree.ps1"),
            "-Keys",
            keys,
            Some(button),
            Duration::from_secs(20),
        )
    }

    fn invoke(&mut self, name: &str) {
        self.script_output(
            &self.sandbox.dir.join("uia_tree.ps1"),
            "-Invoke",
            name,
            None,
            Duration::from_secs(20),
        )
        .unwrap();
    }

    fn control(&mut self, args: &[&str]) {
        self.control_output(args);
    }

    fn control_output(&mut self, args: &[&str]) -> String {
        let capture = self.sandbox.dir.join("control.log");
        let mut command = self
            .sandbox
            .command(env!("CARGO_BIN_EXE_spaceport"), "control.log");
        command.args(args);
        self.command_output(command, &capture, Duration::from_secs(20))
            .unwrap()
    }

    fn daemon_output(&mut self, args: &[&str]) -> String {
        use std::os::windows::process::CommandExt as _;
        let capture = self.sandbox.dir.join("daemon-api.log");
        let mut command = self.sandbox.command(daemon_binary(), "daemon-api.log");
        // CLI children must not open a console and take focus from native input.
        command.creation_flags(sheprd_client::CREATE_NO_WINDOW);
        command.args(args);
        self.command_output(command, &capture, Duration::from_secs(3))
            .unwrap()
    }

    fn daemon_result(&mut self, args: &[&str]) -> serde_json::Value {
        let output = self.daemon_output(args);
        let value: serde_json::Value = serde_json::from_str(&output).unwrap_or_else(|error| {
            panic!(
                "Herdr query {:?} returned {} non-JSON bytes: {error}",
                &args[..args.len().min(2)],
                output.len()
            )
        });
        assert!(value["error"].is_null(), "{value}");
        value["result"].clone()
    }

    fn uia_action(&mut self, flag: &str, name: &str) {
        self.script_output(
            &self.sandbox.dir.join("uia_tree.ps1"),
            flag,
            name,
            None,
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
        .script_output(&script, "-Keys", "", None, Duration::from_millis(100))
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

// keep-sorted start
#[path = "accessibility_windows/announcements.rs"]
mod announcements;
#[path = "accessibility_windows/chat.rs"]
mod chat;
#[path = "accessibility_windows/clipboard_broker.rs"]
mod clipboard_broker;
#[path = "accessibility_windows/clipboard_paste.rs"]
mod clipboard_paste;
#[path = "accessibility_windows/close_others.rs"]
mod close_others;
#[path = "accessibility_windows/core.rs"]
mod core;
#[path = "accessibility_windows/held_shortcuts.rs"]
mod held_shortcuts;
#[path = "accessibility_windows/images.rs"]
mod images;
#[path = "accessibility_windows/keyboard_layouts.rs"]
mod keyboard_layouts;
#[path = "accessibility_windows/non_us.rs"]
mod non_us;
#[path = "accessibility_windows/pane_resize.rs"]
mod pane_resize;
#[path = "accessibility_windows/quit.rs"]
mod quit;
#[path = "accessibility_windows/recovery.rs"]
mod recovery;
#[path = "accessibility_windows/settings.rs"]
mod settings;
#[path = "accessibility_windows/settings_scale.rs"]
mod settings_scale;
#[path = "accessibility_windows/tab_contrast.rs"]
mod tab_contrast;
#[path = "accessibility_windows/terminal.rs"]
mod terminal;
#[path = "accessibility_windows/workspace_close.rs"]
mod workspace_close;
#[path = "accessibility_windows/workspace_reorder.rs"]
mod workspace_reorder;
// keep-sorted end

#[test]
#[ignore = "requires an active Windows desktop and HERDR_TEST_BINARY; launches a native GUI and isolated daemon"]
fn ui_automation_names_the_chrome_and_ctrl_f6_reaches_it() {
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

    let focus = running.read("^{F6}|{TAB}|^{F6}|^{F6}|+{TAB}|{ESC}|^+{F6}");
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
    // Escape and the terminal; Ctrl-Shift-F6 goes back to the last region.
    assert!(moves[5].starts_with("Document 'Terminal'"), "{focus}");
    assert!(!moves[6].starts_with("Document"), "{focus}");
}
