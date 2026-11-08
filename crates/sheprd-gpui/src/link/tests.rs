#![allow(clippy::unwrap_used)]

use super::*;

mod branding;
mod development_scheme;

#[cfg(windows)]
const CHECKOUT: &str = r"C:\Users\me\src\herdr-gpui";
#[cfg(not(windows))]
const CHECKOUT: &str = "/home/me/src/herdr-gpui";

#[test]
fn every_link_reads_back_from_its_url() {
    for link in [
        Link::Open {
            path: CHECKOUT.into(),
        },
        Link::Open {
            path: format!("{CHECKOUT} with spaces & ünïcode").into(),
        },
        Link::Device {
            id: "ssh:me@box.example".into(),
        },
        Link::Notification {
            tag: "herdr:local:boot-1:w1:p2".into(),
        },
        Link::Notification {
            tag: "herdr-preview:local".into(),
        },
    ] {
        let url = link.to_url();
        assert!(url.starts_with("spaceport://"), "{url}");
        assert_eq!(Link::parse(&url).unwrap(), link, "{url}");
    }
}

#[test]
fn browsers_percent_encoding_is_accepted() {
    let encoded = CHECKOUT.replace('\\', "%5C").replace(':', "%3A");
    assert_eq!(
        Link::parse(&format!("spaceport://open?path={encoded}")).unwrap(),
        Link::Open {
            path: CHECKOUT.into()
        }
    );
    assert_eq!(
        Link::parse("spaceport://device/?id=local").unwrap(),
        Link::Device { id: "local".into() }
    );
}

#[test]
fn only_the_known_shapes_are_accepted() {
    let open = |query: &str| Link::parse(&format!("spaceport://open?{query}"));
    let path = format!("path={CHECKOUT}");
    assert_eq!(open(""), Err(Error::MissingParameter("path")));
    assert_eq!(
        open(&format!("{path}&{path}")),
        Err(Error::DuplicateParameter("path"))
    );
    assert_eq!(
        open(&format!("{path}&run=calc")),
        Err(Error::UnknownParameter("run".into()))
    );
    assert_eq!(
        open(&format!("{path}&id=local")),
        Err(Error::UnknownParameter("id".into()))
    );
    assert_eq!(
        Link::parse("spaceport://run?command=calc"),
        Err(Error::UnknownAction("run".into()))
    );
    assert_eq!(Link::parse("https://open?path=/tmp"), Err(Error::Scheme));
    for extra in [
        "spaceport://user@open?path=/x",
        "spaceport://open:80?path=/x",
        "spaceport://open/more?path=/x",
        "spaceport://open?path=/x#frag",
    ] {
        assert_eq!(Link::parse(extra), Err(Error::UnexpectedPart), "{extra}");
    }
    assert!(matches!(
        Link::parse("spaceport://open?path=%"),
        Err(Error::Path)
    ));
    assert_eq!(
        Link::parse(&format!("spaceport://open?path={}", "a".repeat(5000))),
        Err(Error::TooLong)
    );
}

#[test]
fn paths_must_be_local_absolute_and_plain() {
    let parse = |path: &str| local_path(path);
    assert_eq!(parse(CHECKOUT).unwrap(), PathBuf::from(CHECKOUT));
    let mut refused = vec![
        "",
        "relative/dir",
        "/tmp/../etc",
        r"\\server\share",
        "//server/share",
        r"\\?\C:\Windows",
        r"\\.\pipe\x",
    ];
    let control = format!("{CHECKOUT}\nnext");
    refused.push(&control);
    let long = format!("{CHECKOUT}{}", "a".repeat(MAX_PATH));
    refused.push(&long);
    #[cfg(windows)]
    refused.extend(["C:relative", r"C:\dir\file:stream", r"C:\a|b", r"\rooted"]);
    for path in refused {
        assert_eq!(parse(path), Err(Error::Path), "{path:?}");
    }
}

#[test]
fn devices_and_tags_are_bounded_text() {
    assert_eq!(
        Link::parse("spaceport://device?id=a%20b"),
        Err(Error::Device)
    );
    assert_eq!(
        Link::parse(&format!("spaceport://device?id={}", "a".repeat(300))),
        Err(Error::Device)
    );
    assert_eq!(
        Link::parse("spaceport://notification?tag=other"),
        Err(Error::Tag)
    );
    assert_eq!(
        Link::parse("spaceport://notification?tag=herdr:%E2%80%AEx"),
        Err(Error::Tag)
    );
}

#[test]
fn links_are_told_apart_from_options() {
    use std::ffi::OsStr;
    assert!(is_link(OsStr::new("spaceport://open?path=/x")));
    assert!(is_link(OsStr::new("SPACEPORT:whatever")));
    assert!(!is_link(OsStr::new("spaceport")));
    assert!(!is_link(OsStr::new("--socket")));
    assert!(!is_link(OsStr::new("spaceportx:")));
}
