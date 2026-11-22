use super::*;

fn destination(host: &str) -> Destination {
    Destination {
        user: "penso".into(),
        host: host.into(),
        port: 22,
    }
}

#[test]
fn a_claim_blocks_a_second_claim_until_released() -> Result<()> {
    let first = Claim::acquire(destination("claim.test"), "default")?;
    assert!(matches!(
        Claim::acquire(destination("claim.test"), "default"),
        Err(crate::Error::Devices(Error::Busy))
    ));
    // Another session on the same host is a different device.
    drop(Claim::acquire(destination("claim.test"), "work")?);
    drop(first);
    drop(Claim::acquire(destination("claim.test"), "default")?);
    Ok(())
}

#[test]
fn an_empty_label_names_the_device_after_its_target() -> Result<()> {
    assert_eq!(device_label("  ", "penso@10.0.0.9")?, "penso@10.0.0.9");
    assert_eq!(device_label("", "ssh://penso@box:22")?, "penso@box:22");
    assert_eq!(device_label(" Work box ", "box")?, "Work box");
    // A long target is cut to Herdr's limit on a character boundary.
    let long = format!("u@{}", "é".repeat(100));
    let label = device_label("", &long)?;
    assert!(label.len() <= LABEL_LIMIT && long.starts_with(&label));
    assert!(device_label("bad\u{7}", "box").is_err());
    assert!(device_label(&"x".repeat(129), "box").is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn rename_passes_the_label_as_one_value_even_when_it_looks_like_a_flag() -> Result<()> {
    let root = std::env::temp_dir().join(format!("herdr-device-rename-{}", std::process::id()));
    std::fs::create_dir_all(&root)?;
    let binary = root.join("herdr");
    let log = root.join("log");
    crate::test_executable::write(
        &binary,
        format!(
            "#!/bin/sh\nprintf '%s|' \"$@\" > '{}'\n[ \"$3\" = missing ] && {{ echo 'machine profile missing was not found' >&2; exit 1; }}\nexit 0\n",
            log.display()
        ),
        0o700,
    )?;
    let id = "0123456789abcdef0123456789abcdef";
    rename_with(binary.as_os_str(), id, "--work = box")?;
    assert_eq!(
        std::fs::read_to_string(&log)?,
        format!("machine|rename|{id}|--label=--work = box|")
    );
    assert!(matches!(
        rename_with(binary.as_os_str(), "../x", "x"),
        Err(crate::Error::Devices(Error::NoProfile))
    ));
    std::fs::remove_dir_all(&root)?;
    Ok(())
}

#[test]
fn cli_failures_keep_only_the_final_diagnostic_line() {
    assert_eq!(
        last_line(b"connecting\nerror: remote server is not ready\n\n"),
        "error: remote server is not ready"
    );
    assert_eq!(last_line(b"\x1b[31mbad\x1b[0m"), "[31mbad[0m");
    assert_eq!(last_line(&[b'x'; 1000]).len(), 300);
    assert_eq!(last_line(b""), "");
}
