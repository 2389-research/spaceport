use super::*;

#[test]
fn entries_are_device_links_with_safe_short_titles() {
    let entries = entries(&[
        ("local".into(), "Local".into()),
        ("ssh:box".into(), format!("evil\u{202e}{}", "x".repeat(200))),
    ]);
    assert_eq!(entries.devices[0].link, Link::Device { id: "local".into() });
    assert!(
        entries
            .devices
            .iter()
            .all(|item| !item.title.contains('\u{202e}'))
    );
    assert!(
        entries
            .devices
            .iter()
            .all(|item| item.title.chars().count() <= MAX_LABEL)
    );
    assert_eq!(
        entries.devices[1].link,
        Link::Device {
            id: "ssh:box".into()
        }
    );
}

#[test]
fn windows_with_different_device_orders_publish_the_same_list() {
    let local = ("local".to_owned(), "Local".to_owned());
    let boat = ("ssh:boat".to_owned(), "boat".to_owned());
    let alpha = ("ssh:alpha".to_owned(), "Alpha".to_owned());
    let one = entries(&[boat.clone(), local.clone(), alpha.clone()]);
    let other = entries(&[alpha, boat, local]);
    assert_eq!(one, other);
    let titles: Vec<&str> = one.devices.iter().map(|item| item.title.as_str()).collect();
    assert_eq!(titles, ["Show Local", "Show Alpha", "Show boat"]);
}

#[test]
fn refused_entries_are_sent_again_once_the_retry_is_due() {
    let now = Instant::now();
    let list = entries(&[("local".into(), "Local".into())]);
    assert!(due(None, &list, None, now));
    assert!(!due(Some(&list), &list, None, now));
    assert!(!due(Some(&list), &list, Some(now + RETRY), now));
    assert!(due(Some(&list), &list, Some(now), now));
    let other = entries(&[("ssh:box".into(), "Box".into())]);
    assert!(due(Some(&list), &other, Some(now + RETRY), now));
}
