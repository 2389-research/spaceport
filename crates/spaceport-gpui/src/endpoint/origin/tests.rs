use super::*;

#[test]
fn every_mark_names_what_it_stands_for() {
    for origin in [Origin::Local, Origin::Ssh, Origin::Tailscale, Origin::Wsl] {
        assert!(!describe(origin).is_empty(), "{origin:?}");
    }
}
