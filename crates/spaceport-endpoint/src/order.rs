//! The order the window lists devices in. The catalogs list them as each was
//! saved: SSH hosts as upstream keeps them, then WSL distributions. Dragging a
//! device's header reorders them for this client only, by endpoint ID, so
//! upstream's catalog is never rewritten and a device saved later simply lists
//! after the ones the user arranged.

use spaceport_reorder::Beside;

/// Where `id` sorts under the user's `order`: its place there, or after
/// every arranged device, so a stable sort keeps the catalog's own order
/// for the rest.
pub fn rank(order: &[String], id: &str) -> usize {
    order
        .iter()
        .position(|arranged| arranged == id)
        .unwrap_or(order.len())
}

/// The device order after moving `moved` beside an anchor, from `ids`, the
/// devices in their current order. `None` when either is not listed.
pub fn moved(ids: &[&str], moved: &str, to: Beside<&str>) -> Option<Vec<String>> {
    let mut order: Vec<_> = ids.iter().copied().filter(|id| *id != moved).collect();
    if order.len() == ids.len() {
        return None;
    }
    let at = |anchor: &str| order.iter().position(|id| *id == anchor);
    let index = match to {
        Beside::Before(anchor) => at(anchor)?,
        Beside::After(anchor) => at(anchor)? + 1,
    };
    order.insert(index, moved);
    Some(order.into_iter().map(str::to_owned).collect())
}
