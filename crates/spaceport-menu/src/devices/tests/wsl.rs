use super::super::wsl::{Distro, stepped};

/// Ubuntu, Arch and Debian, in that order, with `added` already devices.
fn distros(added: &[&str]) -> Vec<Distro> {
    ["Ubuntu", "Arch", "Debian"]
        .into_iter()
        .map(|name| Distro {
            name: name.to_owned(),
            added: added.contains(&name),
            chosen: false,
        })
        .collect()
}

#[test]
fn the_arrows_step_only_through_distributions_that_can_be_added() {
    let rows = distros(&["Ubuntu"]);
    // With none chosen yet, forward starts at the first one it can add and
    // back at the last.
    assert_eq!(stepped(&rows, None, true), Some("Arch"));
    assert_eq!(stepped(&rows, None, false), Some("Debian"));
    // Either way it wraps, stepping over the distribution already saved.
    assert_eq!(stepped(&rows, Some("Arch"), true), Some("Debian"));
    assert_eq!(stepped(&rows, Some("Debian"), true), Some("Arch"));
    assert_eq!(stepped(&rows, Some("Arch"), false), Some("Debian"));
    assert_eq!(stepped(&rows, Some("Debian"), false), Some("Arch"));
    // A chosen distribution that has since been saved steps on from where it
    // sat, rather than nowhere.
    assert_eq!(stepped(&rows, Some("Ubuntu"), true), Some("Arch"));
}

#[test]
fn nothing_steps_where_nothing_can_be_added() {
    let saved = distros(&["Ubuntu", "Arch", "Debian"]);
    assert_eq!(stepped(&saved, None, true), None);
    assert_eq!(stepped(&[], None, false), None);
}
