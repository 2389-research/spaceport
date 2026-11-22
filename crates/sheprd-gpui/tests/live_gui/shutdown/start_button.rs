// ABOUTME: Clicks the painted Start Herdr button on the owned, focused Xvfb window.
// ABOUTME: Checks the stopped Local view and pointer destination before sending one XTest click.
use super::*;
use display::Display;

#[path = "start_button/tsv.rs"]
mod tsv;

fn xdotool(desktop: &Display, test: &Qualification, args: &[&str]) -> String {
    let sandbox = &test.isolated.sandbox;
    let mut command = sandbox.command("/usr/bin/xdotool", "input.log");
    desktop.configure(&mut command);
    let (status, output) = run(sandbox, command.args(args));
    assert!(status.success(), "xdotool {args:?} failed: {status}");
    output
}

fn field(report: &str, name: &str) -> i64 {
    let mut values = report
        .lines()
        .filter_map(|line| line.split_once('='))
        .filter_map(|(key, value)| (key == name).then_some(value));
    let value = values
        .next()
        .expect("missing X11 geometry field")
        .parse()
        .unwrap();
    assert!(
        values.next().is_none(),
        "duplicate X11 geometry field {name}"
    );
    value
}

fn focused_stopped(test: &mut Qualification) {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let window = test.state().expect("owned window disappeared");
        assert!(
            window.workspace.is_none()
                && window.hosts.iter().any(|host| host.id == "local"
                    && host.selected
                    && host.status == "Herdr stopped"),
            "Local window left stopped state before input"
        );
        assert_eq!((window.width, window.height), (640.0, 600.0));
        if window.active {
            test.record("focused-stopped.json", serde_json::to_vec(&window).unwrap());
            return;
        }
        assert!(
            Instant::now() < deadline,
            "owned GUI did not report active focus"
        );
        thread::sleep(POLL);
    }
}

pub(super) fn click(desktop: &Display, test: &mut Qualification, window: &str) {
    assert_eq!(desktop.window(&test.isolated), window);
    let xid = i64::from_str_radix(window.strip_prefix("0x").unwrap(), 16).unwrap();
    xdotool(desktop, test, &["windowfocus", "--sync", window]);
    focused_stopped(test);

    // Recapture after focus and after the replacement server starts.
    assert!(
        desktop.capture(test, window, "start-button", 640, true),
        "stopped card not painted before input"
    );
    let screenshot = test.evidence.join("start-button.png");
    let sandbox = &test.isolated.sandbox;
    let mut read = sandbox.command("/usr/bin/tesseract", "ocr.log");
    desktop.configure(&mut read);
    let (status, words) = run(
        sandbox,
        read.arg(&screenshot)
            .args(["stdout", "-l", "eng", "--psm", "11", "tsv"]),
    );
    assert!(status.success(), "Tesseract TSV failed: {status}");
    test.record("start-button.tsv", &words);
    let size = image::image_dimensions(&screenshot).unwrap();
    assert_eq!(size, (640, 600));
    let (x, y) = tsv::locate(&words, size).unwrap();
    focused_stopped(test);
    assert_eq!(desktop.window(&test.isolated), window);

    xdotool(
        desktop,
        test,
        &[
            "mousemove",
            "--window",
            window,
            &x.to_string(),
            &y.to_string(),
        ],
    );
    focused_stopped(test);
    assert_eq!(desktop.window(&test.isolated), window);
    let focus = xdotool(desktop, test, &["getwindowfocus", "-f"]);
    assert_eq!(focus.trim().parse::<i64>().unwrap(), xid);
    let geometry = xdotool(desktop, test, &["getwindowgeometry", "--shell", window]);
    let pointer = xdotool(desktop, test, &["getmouselocation", "--shell"]);
    assert_eq!(field(&geometry, "WINDOW"), xid);
    assert_eq!(
        (field(&geometry, "WIDTH"), field(&geometry, "HEIGHT")),
        (640, 600)
    );
    assert_eq!(field(&pointer, "WINDOW"), xid);
    assert_eq!(field(&pointer, "SCREEN"), field(&geometry, "SCREEN"));
    assert_eq!(field(&pointer, "X") - field(&geometry, "X"), i64::from(x));
    assert_eq!(field(&pointer, "Y") - field(&geometry, "Y"), i64::from(y));

    // With an empty window stack, an untargeted click uses XTest at the private pointer.
    xdotool(desktop, test, &["click", "1"]);
    test.record(
        "start-button-click.txt",
        format!("GUI PID {}, window {}, focused XID {window}, button 1 at window ({x}, {y})\nGeometry\n{geometry}Pointer\n{pointer}",
                test.isolated.gui.as_ref().unwrap().id(), test.window.unwrap()),
    );
}
