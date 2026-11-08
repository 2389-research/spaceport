use super::*;

#[test]
fn development_links_round_trip_exact_targets_under_spaceport() {
    let link = Link::Device {
        id: "ssh:box.example".into(),
    };
    let url = link.to_url();
    assert_eq!(url, "spaceport://device?id=ssh%3Abox.example");
    assert_eq!(Link::parse(&url).unwrap(), link);
    assert!(is_link(std::ffi::OsStr::new(&url)));
}

#[test]
fn inherited_link_schemes_are_not_claimed() {
    for scheme in ["sheprd", "herdr-gpui"] {
        let url = format!("{scheme}://device?id=local");
        assert!(!is_link(std::ffi::OsStr::new(&url)));
        assert_eq!(Link::parse(&url), Err(Error::Scheme));
    }
}
