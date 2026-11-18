// ABOUTME: Bounds clipboard reads shared by fields and terminal panes.
// ABOUTME: Holds each read permit until its background acquisition actually exits.
use super::{ClipboardItem, Error, Mode};
use gpui::{App, Global, Task};
use std::sync::{Arc, Weak};

pub(super) const MAX_READS: usize = 4;

#[derive(Default)]
struct Reads(Vec<Weak<()>>);
impl Global for Reads {}

pub(crate) struct Permit(Arc<()>);

pub(crate) fn reserve(cx: &mut App) -> crate::Result<Permit> {
    cx.default_global::<Reads>();
    let reads = cx.global_mut::<Reads>();
    reads.0.retain(|read| read.strong_count() > 0);
    if reads.0.len() >= MAX_READS {
        return Err(Error::Busy.into());
    }
    let lease = Arc::new(());
    reads.0.push(Arc::downgrade(&lease));
    Ok(Permit(lease))
}

pub(crate) fn start(
    mode: Mode,
    cx: &mut App,
) -> crate::Result<Task<crate::Result<Option<ClipboardItem>>>> {
    #[cfg(test)]
    let item = cx.read_from_clipboard();
    start_with(cx, move || {
        #[cfg(test)]
        {
            let _ = mode;
            Ok(item)
        }
        #[cfg(not(test))]
        super::read(mode)
    })
}

pub(crate) fn start_with(
    cx: &mut App,
    read: impl FnOnce() -> crate::Result<Option<ClipboardItem>> + Send + 'static,
) -> crate::Result<Task<crate::Result<Option<ClipboardItem>>>> {
    let permit = reserve(cx)?;
    Ok(cx.background_executor().spawn(async move {
        // Keep the lease in the running task even if the UI stops awaiting it.
        let result = read();
        drop(permit.0);
        result
    }))
}
