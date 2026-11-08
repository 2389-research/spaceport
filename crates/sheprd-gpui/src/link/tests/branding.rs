use super::*;

#[test]
fn development_links_use_the_same_strict_parser_in_any_case() {
    for scheme in ["spaceport", "SPACEPORT"] {
        let text = format!("{scheme}://device?id=local");
        assert!(is_link(std::ffi::OsStr::new(&text)));
        let link = Link::parse(&text).unwrap();
        assert_eq!(link, Link::Device { id: "local".into() });
        assert_eq!(link.to_url(), "spaceport://device?id=local");
        assert_eq!(
            Link::parse(&format!("{scheme}://device?id=local&id=other")),
            Err(Error::DuplicateParameter("id"))
        );
        assert_eq!(
            Link::parse(&format!("{scheme}://notification?tag=herdr:local:pane")),
            Ok(Link::Notification {
                tag: "herdr:local:pane".into()
            })
        );
    }
    for scheme in ["spaceportx", "spaceports"] {
        assert!(!is_link(std::ffi::OsStr::new(&format!("{scheme}:"))));
        assert_eq!(
            Link::parse(&format!("{scheme}://device?id=local")),
            Err(Error::Scheme)
        );
    }
}
