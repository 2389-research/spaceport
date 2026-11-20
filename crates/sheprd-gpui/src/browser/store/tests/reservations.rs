// ABOUTME: Pins ownership of tab identities and slots before asynchronous runtime work.
// ABOUTME: Ensures unrelated opens cannot consume reserved placement capacity.
use super::*;

#[test]
fn reserved_capacity_survives_intervening_opens_and_places_the_original_id() {
    let mut store = Store::default();
    let reserved = store.reserve().unwrap();
    let expected = reserved.id;
    for _ in 1..MAX_TABS {
        assert!(
            store
                .open(Scope::endpoint("local"), "w", Some(Location::Devices), None)
                .is_some()
        );
    }
    assert!(
        store
            .open(Scope::endpoint("local"), "w", Some(Location::Devices), None)
            .is_none()
    );
    assert_eq!(
        store.place_reserved(
            reserved,
            Scope::endpoint("local"),
            "w".into(),
            Location::Devices
        ),
        Some(expected)
    );
    assert!(store.get(expected).is_some());
    assert_eq!(store.reservations.load(Ordering::Relaxed), 0);
}

#[test]
fn dropping_a_reservation_releases_capacity_without_reusing_its_id() {
    let mut store = Store::default();
    let reserved = store.reserve().unwrap();
    let abandoned = reserved.id;
    drop(reserved);
    for _ in 0..MAX_TABS {
        let opened = store
            .open(Scope::endpoint("local"), "w", Some(Location::Devices), None)
            .unwrap();
        assert!(opened > abandoned);
    }
    assert!(matches!(
        store.reserve(),
        Err(crate::browser::Error::Capacity)
    ));
}
