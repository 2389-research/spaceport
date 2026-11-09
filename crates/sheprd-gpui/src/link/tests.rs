#![allow(clippy::unwrap_used)]

use super::*;

mod branding;
mod development_scheme;

#[test]
fn checkout_links_are_refused_while_device_and_notification_links_remain() {
    assert_eq!(
        Link::parse("spaceport://open?path=/tmp/repo"),
        Err(Error::UnknownAction("open".into()))
    );
    assert_eq!(
        Link::parse("spaceport://device?id=local"),
        Ok(Link::Device { id: "local".into() })
    );
    assert_eq!(
        Link::parse("spaceport://notification?tag=herdr-preview:local"),
        Ok(Link::Notification {
            tag: "herdr-preview:local".into()
        })
    );
}

#[test]
fn every_link_reads_back_from_its_url() {
    for link in [
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
    assert_eq!(
        Link::parse("spaceport://device/?id=local").unwrap(),
        Link::Device { id: "local".into() }
    );
    assert_eq!(
        Link::parse("spaceport://notification?tag=herdr%3Alocal%3Aw1").unwrap(),
        Link::Notification {
            tag: "herdr:local:w1".into()
        }
    );
}

#[test]
fn only_the_known_shapes_are_accepted() {
    assert_eq!(
        Link::parse("spaceport://device"),
        Err(Error::MissingParameter("id"))
    );
    assert_eq!(
        Link::parse("spaceport://device?id=local&id=other"),
        Err(Error::DuplicateParameter("id"))
    );
    assert_eq!(
        Link::parse("spaceport://device?id=local&path=/tmp"),
        Err(Error::UnknownParameter("path".into()))
    );
    assert_eq!(
        Link::parse("spaceport://run?command=calc"),
        Err(Error::UnknownAction("run".into()))
    );
    assert_eq!(Link::parse("https://device?id=local"), Err(Error::Scheme));
    for extra in [
        "spaceport://user@device?id=local",
        "spaceport://device:80?id=local",
        "spaceport://device/more?id=local",
        "spaceport://device?id=local#frag",
    ] {
        assert_eq!(Link::parse(extra), Err(Error::UnexpectedPart), "{extra}");
    }
    assert_eq!(
        Link::parse(&format!("spaceport://device?id={}", "a".repeat(5000))),
        Err(Error::TooLong)
    );
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
    assert!(is_link(OsStr::new("spaceport://device?id=local")));
    assert!(is_link(OsStr::new("SPACEPORT:whatever")));
    assert!(!is_link(OsStr::new("spaceport")));
    assert!(!is_link(OsStr::new("--socket")));
    assert!(!is_link(OsStr::new("spaceportx:")));
}
