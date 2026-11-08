use super::*;

#[test]
fn machine_names_follow_each_providers_argument_contract() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(FILE);
    for &provider in CloudProvider::ALL {
        for name in ["-oProxyCommand=x", "a b"] {
            let mut saved = device("w");
            saved.provider = provider;
            saved.machine = name.into();
            assert!(
                write(&path, vec![Entry::Known(saved)]).is_err(),
                "{provider:?}: {name:?}"
            );
        }
    }
}
