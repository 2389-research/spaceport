// ABOUTME: Owns a cookie-protected Xvfb display and captures the named Spaceport process's visible window.
// ABOUTME: Keeps transient warning frames and reads their painted text after both widths are captured.
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

    pub(super) fn await_size(&self, isolated: &Isolated, window: &str, width: u32) {
        assert_eq!(self.window(isolated), window);
        let deadline = Instant::now() + TIMEOUT;
        let expected = format!("WIDTH={width}");
        loop {
            let mut command = isolated.sandbox.command("/usr/bin/xdotool", "capture.log");
            self.configure(&mut command);
            let (status, geometry) = run(
                &isolated.sandbox,
                command.args(["getwindowgeometry", "--shell", window]),
            );
            assert!(status.success(), "reading native window geometry failed");
            if geometry.lines().any(|line| line == expected)
                && geometry.lines().any(|line| line == "HEIGHT=600")
            {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "expected {width}x600; got {geometry}"
            );
            thread::sleep(POLL);
        }
    }

    pub(super) fn capture(&self, test: &Qualification, window: &str, width: u32) -> Vec<PathBuf> {
        assert_eq!(self.window(&test.isolated), window);
        let sandbox = &test.isolated.sandbox;
        let mut frames = Vec::new();
        // OCR can outlast a flash. Take several frames through its entrance first.
        for index in 0..4 {
            let path = test
                .evidence
                .join(format!("repair-{width}-frame-{index}.png"));
            let mut capture = sandbox.command("/usr/bin/import", "capture.log");
            self.configure(&mut capture);
            let (status, _) = run(
                sandbox,
                capture.args(["-silent", "-window", window]).arg(&path),
            );
            assert!(status.success(), "native capture failed");
            assert_eq!(image::image_dimensions(&path).unwrap(), (width, 600));
            frames.push(path);
            thread::sleep(POLL);
        }
        assert_eq!(self.window(&test.isolated), window);
        frames
    }

    pub(super) fn best_frame(
        &self,
        test: &Qualification,
        width: u32,
        frames: &[PathBuf],
        score: impl Fn(&str) -> (bool, usize),
    ) -> bool {
        let mut best = None;
        for frame in frames {
            let mut read = test
                .isolated
                .sandbox
                .command("/usr/bin/tesseract", "ocr.log");
            self.configure(&mut read);
            let (status, text) = run(
                &test.isolated.sandbox,
                read.arg(frame).args(["stdout", "-l", "eng", "--psm", "11"]),
            );
            assert!(status.success(), "reading native warning text failed");
            let rank = score(&text);
            if best.is_none_or(|previous| rank >= previous) {
                fs::copy(frame, test.evidence.join(format!("repair-{width}.png"))).unwrap();
                test.record(&format!("repair-{width}-text.txt"), &text);
                best = Some(rank);
            }
        }
        best.unwrap().0
    }

    pub(super) fn stop(mut self, test: &Qualification) {
        let child = self.server.as_mut().unwrap();
        assert!(child.try_wait().unwrap().is_none());
        child.kill().unwrap();
        let status = wait(child, "private Xvfb");
        test.record("display-exit.txt", format!("{} {status}\n", child.id()));
        self.server = None;
    }
}
