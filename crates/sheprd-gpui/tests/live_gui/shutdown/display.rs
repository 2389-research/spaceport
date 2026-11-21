// ABOUTME: Owns a cookie-protected Xvfb display and captures the named Spaceport process's visible window.
// ABOUTME: Checks actual PNG dimensions and reads card text from painted pixels.
use super::*;
use std::os::unix::fs::OpenOptionsExt;

pub(super) struct Display {
    server: Option<Child>,
    address: String,
    authority: PathBuf,
}

impl Drop for Display {
    fn drop(&mut self) {
        stop_children([&mut self.server]);
    }
}

impl Display {
    pub(super) fn start(sandbox: &Sandbox) -> Self {
        let authority = sandbox.dir.join("Xauthority");
        // Xvfb chooses its free display number after startup. FamilyWild and an
        // empty display number let Xlib use this one cookie for that allocation.
        let mut auth = File::options()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&authority)
            .unwrap();
        let mut cookie = [0; 16];
        File::open("/dev/urandom")
            .unwrap()
            .read_exact(&mut cookie)
            .unwrap();
        auth.write_all(&u16::MAX.to_be_bytes()).unwrap();
        for field in [&[][..], &[][..], b"MIT-MAGIC-COOKIE-1", &cookie] {
            auth.write_all(&(field.len() as u16).to_be_bytes()).unwrap();
            auth.write_all(field).unwrap();
        }
        drop(auth);
        let ready = sandbox.dir.join("display-number");
        let server = sandbox
            .command("/usr/bin/Xvfb", "xvfb.log")
            .args([
                "-displayfd",
                "1",
                "-screen",
                "0",
                "1280x900x24",
                "-dpi",
                "96",
                "-nolisten",
                "tcp",
                "-noreset",
                "-auth",
            ])
            .arg(&authority)
            .stdout(File::create(&ready).unwrap())
            .spawn()
            .unwrap();
        let mut display = Self {
            server: Some(server),
            address: String::new(),
            authority,
        };
        let deadline = Instant::now() + TIMEOUT;
        loop {
            assert!(
                display
                    .server
                    .as_mut()
                    .unwrap()
                    .try_wait()
                    .unwrap()
                    .is_none(),
                "Xvfb exited"
            );
            let number = fs::read_to_string(&ready).unwrap();
            if let Some(number) = number.strip_suffix('\n') {
                display.address = format!(":{}", number.parse::<u16>().unwrap());
                return display;
            }
            assert!(
                Instant::now() < deadline,
                "Xvfb did not publish its display number"
            );
            thread::sleep(POLL);
        }
    }

    pub(super) fn configure(&self, command: &mut Command) {
        command
            .env("DISPLAY", &self.address)
            .env("XAUTHORITY", &self.authority);
    }

    pub(super) fn window(&self, isolated: &Isolated) -> String {
        let mut command = isolated.sandbox.command("/usr/bin/xdotool", "capture.log");
        self.configure(&mut command);
        let (status, output) = run(
            &isolated.sandbox,
            command.args([
                "search",
                "--onlyvisible",
                "--pid",
                &isolated.gui.as_ref().unwrap().id().to_string(),
            ]),
        );
        assert!(status.success(), "no native window for the owned GUI");
        let id = output
            .trim()
            .parse::<u32>()
            .expect("expected one visible X window for the GUI PID");
        format!("0x{id:x}")
    }

    pub(super) fn capture(
        &self,
        test: &Qualification,
        window: &str,
        label: &str,
        width: u32,
        stopped: bool,
    ) -> bool {
        assert_eq!(self.window(&test.isolated), window);
        let sandbox = &test.isolated.sandbox;
        let path = test.evidence.join(format!("{label}.png"));
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let mut capture = sandbox.command("/usr/bin/import", "capture.log");
            self.configure(&mut capture);
            let (status, _) = run(
                sandbox,
                capture.args(["-silent", "-window", window]).arg(&path),
            );
            assert!(status.success(), "native capture failed");
            let size = image::image_dimensions(&path).unwrap();
            // A resize or reconnect can be acknowledged before the window repaints.
            if size == (width, 600) {
                let mut read = sandbox.command("/usr/bin/tesseract", "ocr.log");
                let (status, text) = run(
                    sandbox,
                    read.arg(&path).args(["stdout", "-l", "eng", "--psm", "11"]),
                );
                assert!(status.success(), "reading native card text failed");
                test.record(&format!("{label}-text.txt"), &text);
                let text = text
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .to_lowercase();
                let title = text.contains("herdr stopped");
                let action = text.contains("start herdr");
                let painted = (stopped && title && action)
                    || (!stopped
                        && text.contains(&REPLACEMENT_PROMPT.to_lowercase())
                        && !title
                        && !action
                        && !text.contains("reconnect now")
                        && !text.contains("reconnecting to"));
                if painted || Instant::now() >= deadline {
                    assert!(!text.is_empty(), "{label}: OCR returned no visible text");
                    assert_eq!(self.window(&test.isolated), window);
                    test.record(
                        &format!("{label}-window.txt"),
                        format!("XID {window}, {width}x600\n"),
                    );
                    return painted;
                }
            } else {
                assert!(
                    Instant::now() < deadline,
                    "{label}: expected native dimensions {width}x600; captured {size:?}"
                );
            }
            thread::sleep(POLL);
        }
    }
}
