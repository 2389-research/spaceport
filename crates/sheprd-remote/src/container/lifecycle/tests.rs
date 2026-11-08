use super::*;

#[test]
fn starts_of_the_same_saved_device_share_one_lock() {
    let first = device("docker", "saved-one");
    assert!(Arc::ptr_eq(&first, &device("docker", "saved-one")));
    assert!(!Arc::ptr_eq(&first, &device("podman", "saved-one")));
    assert!(!Arc::ptr_eq(&first, &device("docker", "saved-two")));
}
