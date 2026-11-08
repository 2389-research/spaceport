use super::*;
use crate::link::Link;

#[test]
fn cli_hands_a_spaceport_link_to_the_exact_device() {
    let options = LaunchOptions::parse(["spaceport://device?id=ssh%3Abox.example"]).unwrap();
    assert_eq!(options.target, ConnectTarget::Local);
    assert_eq!(
        options.mode,
        LaunchMode::Link(Link::Device {
            id: "ssh:box.example".into()
        })
    );
}
