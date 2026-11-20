// ABOUTME: Verifies bounded Wayland and X11 clipboard acquisition.
// ABOUTME: Uses isolated data and exact helper processes.
use super::*;

mod current_selection;

#[test]
fn commands_preserve_text_and_skip_image_requests() -> Result<()> {
    let item = read_commands(Mode::Normal, true, true, |program, args, limit, _| {
        assert_eq!(program, "wl-paste");
        assert_eq!(args, ["--no-newline", "--type", "text/plain;charset=utf-8"]);
        assert_eq!(limit, TEXT_LIMIT);
        Ok(Some(b"text\n".to_vec()))
    })?;
    assert_eq!(item.and_then(|item| item.text()).as_deref(), Some("text\n"));
    Ok(())
}

#[test]
fn large_output_arrives_before_the_deadline() -> Result<()> {
    // A socket pair holds a few kilobytes; sleeping between reads
    // took far longer than the deadline for an image this size.
    const SIZE: usize = 26 * 1024 * 1024;
    let started = Instant::now();
    let deadline = started + Duration::from_secs(3);
    let bytes = output(
        Command::new("/bin/sh").args(["-c", &format!("head -c {SIZE} /dev/zero")]),
        IMAGE_LIMIT,
        deadline,
    )?;
    assert_eq!(bytes.map(|bytes| bytes.len()), Some(SIZE));
    assert!(Instant::now() < deadline);
    Ok(())
}

#[test]
fn image_commands_fall_back_and_reject_text_from_image_targets() -> Result<()> {
    let mut calls = Vec::new();
    let mut first_deadline = None;
    let item = read_commands(
        Mode::Image,
        false,
        true,
        |program, args, limit, deadline| {
            assert_eq!(limit, IMAGE_LIMIT);
            assert_eq!(*first_deadline.get_or_insert(deadline), deadline);
            calls.push(program.to_owned());
            if program == "xclip" && args.contains(&"image/jpg") {
                Ok(Some(vec![0xff, 0xd8, 0xff]))
            } else {
                Ok(Some(b"not an image".to_vec())) // i18n: data
            }
        },
    )?;
    assert_eq!(calls, ["xclip", "xclip", "xclip"]);
    assert!(matches!(item.as_ref().map(|item| item.entries()),
                Some([gpui::ClipboardEntry::Image(image)]) if image.format == ImageFormat::Jpeg));
    assert!(read_commands(Mode::Image, false, false, |_, _, _, _| panic!("no display"))?.is_none());
    Ok(())
}

#[test]
fn process_output_is_bounded_binary_and_separate_from_stderr() -> Result<()> {
    let run = |script, limit| {
        output(
            Command::new("/bin/sh").args(["-c", script]),
            limit,
            Instant::now() + Duration::from_secs(3),
        )
    };
    assert_eq!(
        run("printf '\\377abc'; printf error >&2", 4)?,
        Some(vec![255, b'a', b'b', b'c'])
    );
    assert!(matches!(
        run("printf abcde", 4),
        Err(crate::Error::Clipboard(Error::Size { limit: 4 }))
    ));
    assert!(run("printf partial; exit 1", 16)?.is_none());
    assert_eq!(run("exit 0", 0)?, Some(Vec::new()));
    assert!(matches!(
        output(
            &mut Command::new("/nonexistent/herdr-clipboard-helper"),
            4,
            Instant::now() + Duration::from_secs(3)
        ),
        Err(crate::Error::Clipboard(Error::Process(error)))
            if error.kind() == io::ErrorKind::NotFound
    ));
    Ok(())
}

#[test]
fn timeout_covers_open_and_closed_stdout_and_continuous_output() {
    for script in [
        "exec sleep 30",              // i18n: data
        "exec 1>&-; exec sleep 30",   // i18n: data
        "while :; do printf x; done", // i18n: data
    ] {
        let start = Instant::now();
        let result = output(
            Command::new("/bin/sh").args(["-c", script]),
            usize::MAX,
            start + Duration::from_millis(100),
        );
        assert!(matches!(
            result,
            Err(crate::Error::Clipboard(Error::Timeout))
        ));
        assert!(start.elapsed() < Duration::from_secs(5));
    }
}
