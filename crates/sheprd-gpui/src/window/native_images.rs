// ABOUTME: Observes owned terminal cell and image paints without changing their scene.
// ABOUTME: Keeps native image qualification counters scoped to one fixture window.
use anyhow::{Result, ensure};
use gpui::{Bounds, ImageId, Pixels, WindowId};
use std::cell::RefCell;

mod raster;

#[cfg(all(windows, feature = "integration-test"))]
mod native;
#[cfg(all(windows, feature = "integration-test"))]
mod scene;
#[cfg(all(windows, feature = "integration-test"))]
pub(super) use native::{enabled, observe};

#[derive(Clone, Debug)]
pub(crate) struct ImagePaint {
    pub(crate) id: ImageId,
    pub(crate) image: Bounds<Pixels>,
    pub(crate) mask: Bounds<Pixels>,
    pub(crate) painted: bool,
    pub(crate) visible: Bounds<Pixels>,
    pub(crate) z: i32,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct FramePaint {
    pub(crate) cells: usize,
    pub(crate) images: Vec<ImagePaint>,
    pub(crate) overflow: bool,
}

struct Probe {
    frame: FramePaint,
    window: WindowId,
}

thread_local! {
    static PROBE: RefCell<Option<Probe>> = const { RefCell::new(None) };
}

pub(crate) fn active(window: WindowId) -> bool {
    PROBE.with_borrow(|slot| slot.as_ref().is_some_and(|probe| probe.window == window))
}

pub(crate) fn cells(window: WindowId) {
    PROBE.with_borrow_mut(|slot| {
        if let Some(probe) = slot.as_mut().filter(|probe| probe.window == window) {
            probe.frame.cells += 1;
        }
    });
}

pub(crate) fn image(window: WindowId, image: ImagePaint) {
    PROBE.with_borrow_mut(|slot| {
        if let Some(probe) = slot.as_mut().filter(|probe| probe.window == window) {
            if probe.frame.images.len() < 16 {
                probe.frame.images.push(image);
            } else {
                probe.frame.overflow = true;
            }
        }
    });
}

pub(crate) struct Session(WindowId);

impl Session {
    pub(crate) fn start(window: WindowId) -> Result<Self> {
        PROBE.with_borrow_mut(|slot| {
            ensure!(slot.is_none(), "native image paint probes must not overlap");
            *slot = Some(Probe {
                frame: FramePaint::default(),
                window,
            });
            Ok(Self(window))
        })
    }

    pub(crate) fn reset(&self) -> Result<()> {
        PROBE.with_borrow_mut(|slot| {
            let Some(probe) = slot.as_mut().filter(|probe| probe.window == self.0) else {
                anyhow::bail!("native image paint probe lost its owned window");
            };
            probe.frame = FramePaint::default();
            Ok(())
        })
    }

    pub(crate) fn frame(&self) -> Result<FramePaint> {
        PROBE.with_borrow(|slot| {
            let Some(probe) = slot.as_ref().filter(|probe| probe.window == self.0) else {
                anyhow::bail!("native image paint probe lost its owned window");
            };
            Ok(probe.frame.clone())
        })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        PROBE.with_borrow_mut(|slot| {
            if slot.as_ref().is_some_and(|probe| probe.window == self.0) {
                *slot = None;
            }
        });
    }
}
