use super::*;
use crate::MAX_VOLUME;

#[test]
fn a_fixture_worker_starts_at_full_level_until_a_caller_sets_one() {
    let (service, volumes) = Service::recording_volume();
    service.preview();
    assert_eq!(
        volumes.recv_timeout(Duration::from_secs(3)).unwrap(),
        MAX_VOLUME
    );
}

#[test]
fn each_sound_plays_at_the_volume_set_before_it() {
    let (service, volumes) = Service::recording_volume();
    for percent in [0, 30, 100] {
        service.set_volume(percent);
        service.preview();
        assert_eq!(
            volumes.recv_timeout(Duration::from_secs(3)).unwrap(),
            percent
        );
    }
}
