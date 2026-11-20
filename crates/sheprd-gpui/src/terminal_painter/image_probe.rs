// ABOUTME: Observes actual terminal image uploads and paint epochs in scoped tests.
// ABOUTME: Marks real paint layers so cached scene replay remains inspectable.
use super::{ImageTarget, Layer, PlacedImages, images::ImageGeometry};
use gpui::{Bounds, ImageId, Pixels, RenderImage, Window, fill, rgb};
use std::{cell::RefCell, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Cells { popup: bool, layer: Layer },
    Image { popup: bool, z: i32, id: ImageId },
}

#[derive(Clone)]
pub(crate) struct PaintedImage {
    pub(crate) phase: Phase,
    pub(crate) bounds: Bounds<Pixels>,
    pub(crate) mapped: Bounds<Pixels>,
    pub(crate) texture: Arc<RenderImage>,
    pub(crate) uploaded: bool,
}

#[derive(Default)]
struct Probe {
    popup: bool,
    markers: Vec<Phase>,
    painted: Vec<PaintedImage>,
    preparations: Vec<(bool, u64)>,
    popup_leases: Vec<usize>,
    main_cell_callbacks: usize,
}

thread_local! {
    static PROBE: RefCell<Option<Probe>> = const { RefCell::new(None) };
}

/// A probe belongs to one test thread and releases its observations on drop.
pub(crate) struct Session;

impl Session {
    pub(crate) fn start() -> Self {
        PROBE.with_borrow_mut(|probe| {
            assert!(probe.is_none(), "image probes must not overlap");
            *probe = Some(Probe::default());
        });
        Self
    }

    pub(crate) fn reset(&self) {
        PROBE.with_borrow_mut(|probe| {
            if let Some(probe) = probe {
                probe.painted.clear();
                probe.preparations.clear();
                probe.popup_leases.clear();
                probe.main_cell_callbacks = 0;
            }
        });
    }

    pub(crate) fn painted(&self) -> Vec<PaintedImage> {
        PROBE.with_borrow(|probe| probe.as_ref().map_or_else(Vec::new, |p| p.painted.clone()))
    }

    pub(crate) fn preparations(&self) -> Vec<(bool, u64)> {
        PROBE.with_borrow(|probe| {
            probe
                .as_ref()
                .map_or_else(Vec::new, |p| p.preparations.clone())
        })
    }

    pub(crate) fn popup_leases(&self) -> Vec<usize> {
        PROBE.with_borrow(|probe| {
            probe
                .as_ref()
                .map_or_else(Vec::new, |p| p.popup_leases.clone())
        })
    }

    pub(crate) fn main_cell_callbacks(&self) -> usize {
        PROBE.with_borrow(|probe| probe.as_ref().map_or(0, |p| p.main_cell_callbacks))
    }

    pub(crate) fn scene(&self, window: &Window) -> Vec<(Phase, u32)> {
        let quads = window.painted_quads();
        PROBE.with_borrow(|probe| {
            let Some(probe) = probe else {
                return Vec::new();
            };
            let mut phases: Vec<_> = quads
                .iter()
                .filter_map(|quad| {
                    probe.markers.iter().enumerate().find_map(|(index, phase)| {
                        (quad.background == rgb(marker_colour(index)).into())
                            .then_some((*phase, quad.order))
                    })
                })
                .collect();
            phases.sort_by_key(|(_, order)| *order);
            phases
        })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        PROBE.with_borrow_mut(|probe| *probe = None);
    }
}

fn marker_colour(index: usize) -> u32 {
    0x6c0000 + index as u32
}

fn mark(probe: &mut Probe, phase: Phase, bounds: Bounds<Pixels>, window: &mut Window) {
    let index = probe
        .markers
        .iter()
        .position(|found| *found == phase)
        .unwrap_or_else(|| {
            probe.markers.push(phase);
            probe.markers.len() - 1
        });
    window.paint_quad(fill(bounds, rgb(marker_colour(index))));
}

pub(super) fn cells(
    layer: Layer,
    images: Option<PlacedImages<'_>>,
    bounds: Bounds<Pixels>,
    window: &mut Window,
) {
    PROBE.with_borrow_mut(|probe| {
        if let Some(probe) = probe {
            let popup = images.is_some_and(|images| matches!(images.target, ImageTarget::Popup(_)));
            probe.main_cell_callbacks += usize::from(!popup);
            mark(probe, Phase::Cells { popup, layer }, bounds, window);
        }
    });
}

pub(super) fn prepare(target: ImageTarget<'_>, epoch: u64) {
    PROBE.with_borrow_mut(|probe| {
        if let Some(probe) = probe {
            probe.popup = matches!(target, ImageTarget::Popup(_));
            probe.preparations.push((probe.popup, epoch));
            if probe.popup {
                probe.popup_leases = probe
                    .painted
                    .iter()
                    .map(|image| Arc::strong_count(&image.texture))
                    .collect();
            }
        }
    });
}

pub(super) fn image(
    z: i32,
    geometry: &ImageGeometry,
    texture: &Arc<RenderImage>,
    successful: bool,
    window: &mut Window,
) {
    PROBE.with_borrow_mut(|probe| {
        if let Some(probe) = probe {
            let phase = Phase::Image {
                popup: probe.popup,
                z,
                id: texture.id,
            };
            probe.painted.push(PaintedImage {
                phase,
                bounds: geometry.visible.intersect(&window.content_mask().bounds),
                mapped: geometry.visible,
                texture: texture.clone(),
                uploaded: successful && window.has_image_atlas_entry(texture),
            });
            // This marker follows the actual paint_image call in the same context.
            mark(probe, phase, geometry.visible, window);
        }
    });
}
