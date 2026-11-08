use crate::Host;
use sheprd_client::{CloudProvider, ConnectTarget};

fn container(id: &str) -> ConnectTarget {
    ConnectTarget::Cloud {
        provider: CloudProvider::Container,
        account: "00000000-0000-0000-0000-000000000000".into(),
        id: id.into(),
        machine: "box".into(),
        session: "default".into(),
    }
}

#[test]
fn a_container_has_no_load_or_usage_host() {
    for id in ["one", "two"] {
        let target = container(id);
        assert_eq!(Host::of(&target), None);
        assert_eq!(Host::load_of(&target), None);
    }
}

#[test]
fn other_hosts_load_where_they_are_used() {
    for target in [
        ConnectTarget::Local,
        ConnectTarget::Ssh {
            target: "build".into(),
            session: "default".into(),
        },
    ] {
        assert_eq!(Host::load_of(&target), Host::of(&target));
    }
}
