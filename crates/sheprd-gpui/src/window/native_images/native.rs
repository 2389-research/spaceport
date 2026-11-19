// ABOUTME: Captures real image-bearing terminal frames after native rendering completes.
// ABOUTME: Keeps producer, cached replay, wheel, and settled evidence behind an opt-in fixture token.
use super::{
    FramePaint, Session,
    scene::{self, Scene, Shot},
};
use crate::{SheprdWindow, presentation::Picture, smooth_scroll::Slide, terminal::CellMetrics};
use anyhow::{Result, ensure};
use base64::Engine as _;
use gpui::{App, AppContext as _, AsyncApp, Bounds, Context, Entity, EntityId, Pixels, Window};
use sheprd_client::protocol::PaneSurfaceFrame;
use std::{
    cell::RefCell,
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};

struct Frame {
    assets: usize,
    frame: Option<Arc<PaneSurfaceFrame>>,
    metrics: CellMetrics,
    moving: bool,
    regions: Vec<EntityId>,
    serials: Option<Vec<u64>>,
    slide: Option<Slide>,
}

enum Stage {
    Shell,
    Producer,
    Replay(Box<Shot>),
    Wheel {
        base: Box<Shot>,
        replay: Box<Shot>,
    },
    Settle {
        base: Box<Shot>,
        replay: Box<Shot>,
        moving: Box<Shot>,
    },
    Encoding,
    Done,
}

struct Observer {
    cells_seen: bool,
    deadline: Instant,
    probe: Session,
    producer_frames: usize,
    stage: Stage,
    wheel_frames: usize,
}

thread_local! {
    static OBSERVER: RefCell<Option<Observer>> = const { RefCell::new(None) };
}

fn token() -> Option<uuid::Uuid> {
    static TOKEN: OnceLock<Option<uuid::Uuid>> = OnceLock::new();
    *TOKEN.get_or_init(|| {
        std::env::var_os("HERDR_TEST_NATIVE_IMAGES").map(|value| {
            value
                .to_str()
                .and_then(|value| uuid::Uuid::parse_str(value).ok())
                .unwrap_or_else(|| fail(anyhow::anyhow!("native image fixture token is invalid")))
        })
    })
}

fn fail(error: anyhow::Error) -> ! {
    eprintln!("IMAGE_NATIVE_FAIL {error:#}");
    std::process::exit(1);
}

pub(in crate::window) fn enabled() -> bool {
    token().is_some()
}

pub(in crate::window) fn observe(
    view: &SheprdWindow,
    picture: Option<&Picture>,
    slide: Option<&Slide>,
    window: &Window,
    cx: &mut Context<SheprdWindow>,
) {
    let Some(token) = token() else { return };
    let result = OBSERVER.with_borrow_mut(|slot| -> Result<bool> {
        if slot.is_none() {
            *slot = Some(Observer {
                cells_seen: false,
                deadline: Instant::now() + Duration::from_secs(90),
                probe: Session::start(window.window_handle().window_id())?,
                producer_frames: 0,
                stage: Stage::Shell,
                wheel_frames: 0,
            });
        }
        let Some(observer) = slot else {
            anyhow::bail!("native image observer missing")
        };
        ensure!(
            observer.probe.0 == window.window_handle().window_id(),
            "native image observer encountered another window"
        );
        if matches!(observer.stage, Stage::Encoding | Stage::Done) {
            return Ok(false);
        }
        observer.probe.reset()?;
        Ok(true)
    });
    match result {
        Ok(false) => return,
        Err(error) => fail(error),
        Ok(true) => {}
    }
    let coherent = view.live.surface_ready()
        && !view.presentation.stale()
        && picture.is_some_and(|picture| {
            view.live
                .surface
                .as_ref()
                .is_some_and(|surface| Arc::ptr_eq(surface, &picture.frame))
        });
    let frame = Frame {
        assets: picture.map_or(0, |picture| picture.images.len()),
        frame: coherent
            .then(|| picture.map(|picture| picture.frame.clone()))
            .flatten(),
        metrics: view.metrics(),
        moving: view.presentation.scroll.moving(crate::motion::now()),
        regions: view
            .regions
            .iter()
            .flat_map(|layers| layers.iter().map(Entity::entity_id))
            .collect(),
        serials: coherent
            .then(|| picture.and_then(|picture| scene::authored(picture, token)))
            .flatten(),
        slide: slide.cloned(),
    };
    let entity = cx.entity();
    window.on_next_frame(move |window, cx| {
        if let Err(error) = after_frame(&entity, frame, window, cx) {
            let pixels = window.render_to_image();
            OBSERVER.with_borrow_mut(|slot| {
                if let Some(observer) = slot {
                    observer.stage = Stage::Encoding;
                }
            });
            cx.spawn(async move |cx| {
                match pixels {
                    Ok(pixels) => {
                        if let Err(capture_error) = receipt("failure", pixels, cx).await {
                            eprintln!("IMAGE_NATIVE_FAILURE_CAPTURE_ERROR {capture_error:#}");
                        }
                    }
                    Err(capture_error) => {
                        eprintln!("IMAGE_NATIVE_FAILURE_CAPTURE_ERROR {capture_error:#}")
                    }
                }
                fail(error);
            })
            .detach();
        }
    });
}

fn capture(frame: Frame, paint: FramePaint, pane: Bounds<Pixels>, window: &Window) -> Result<Shot> {
    let pixels = window.render_to_image()?;
    ensure!(
        u64::from(pixels.width()) * u64::from(pixels.height()) <= 2_000_000,
        "native image capture exceeds pixel bound"
    );
    let surface = frame
        .frame
        .ok_or_else(|| anyhow::anyhow!("native image frame lost coherence"))?;
    let serials = frame
        .serials
        .ok_or_else(|| anyhow::anyhow!("authored image assets are missing"))?;
    let viewport = window.viewport_size();
    let scale = pixels.width() as f32 / f32::from(viewport.width);
    ensure!(
        (pixels.height() as f32 / f32::from(viewport.height) - scale).abs() < 0.01,
        "native image capture has inconsistent scaling"
    );
    let shot = Shot {
        pixels,
        paint,
        pane,
        scale,
        viewport,
        scene: Scene {
            frame: surface,
            metrics: frame.metrics,
            regions: frame.regions,
            serials,
        },
    };
    scene::validate_pixels(&shot)?;
    Ok(shot)
}

fn after_frame(
    entity: &Entity<SheprdWindow>,
    frame: Frame,
    window: &Window,
    cx: &mut App,
) -> Result<()> {
    let view = entity.read(cx);
    let focused =
        window.is_window_active() && view.focus.is_focused(window) && view.menu.page.is_none();
    let pane = frame.frame.as_ref().and_then(|surface| {
        let [pane] = surface.panes.as_slice() else {
            return None;
        };
        let bounds = frame.metrics.bounds(pane.inner_rect);
        Some(Bounds::new(view.bounds.origin + bounds.origin, bounds.size))
    });
    let completed = OBSERVER.with_borrow_mut(|slot| -> Result<Option<[Box<Shot>; 4]>> {
        let Some(observer) = slot else { anyhow::bail!("native image observer disappeared") };
        ensure!(Instant::now() < observer.deadline, "native image qualification exceeded deadline");
        let paint = observer.probe.frame()?;
        observer.cells_seen |= paint.cells > 0;
        if !focused { return Ok(None) }
        ensure!(!paint.overflow, "native image paint observations overflowed");
        let stage = std::mem::replace(&mut observer.stage, Stage::Done);
        observer.stage = match stage {
            Stage::Shell => {
                let ready = frame.frame.as_ref().filter(|surface| {
                    surface.panes.len() == 1 && surface.frame.cursor.as_ref().is_some_and(|cursor| cursor.visible)
                        && surface.frame.cells.iter().map(|cell| cell.symbol.as_str()).collect::<String>().contains("PS ")
                });
                if let Some(surface) = ready {
                    let pane = &surface.panes[0];
                    ensure!(pane.inner_rect.width >= 64 && pane.inner_rect.height >= 22,
                        "private image pane is too small for the authored scene");
                    eprintln!("IMAGE_NATIVE_VIEW {}", serde_json::json!({
                        "pane": pane.pane_id, "cols": pane.inner_rect.width, "rows": pane.inner_rect.height
                    }));
                    Stage::Producer
                } else { Stage::Shell }
            }
            Stage::Producer => {
                let ready = frame.serials.is_some() && [-1, 2, 3, 4].iter().all(|z| scene::image(&paint, *z).is_ok());
                if !ready && matches!(observer.producer_frames, 0 | 60 | 300) {
                    eprintln!("IMAGE_NATIVE_PRODUCER_WAIT assets={} placements={} authored={} uploads={}",
                        frame.assets, frame.frame.as_ref().map_or(0, |surface| surface.graphics.placements.len()),
                        frame.serials.is_some(), paint.images.len());
                }
                observer.producer_frames += 1;
                if ready {
                    let shot = capture(frame, paint, pane.ok_or_else(|| anyhow::anyhow!("private image pane missing"))?, window)?;
                    ensure!(observer.cells_seen && !shot.scene.regions.is_empty(),
                        "native image case has no observed cached terminal cell regions");
                    ensure!(shot.scene.frame.panes[0].scroll.is_some_and(|scroll| scroll.max_offset_from_bottom > 0),
                        "authored image scene has no real scrollback");
                    Stage::Replay(Box::new(shot))
                } else { Stage::Producer }
            }
            Stage::Replay(base) => {
                ensure!(paint.cells == 0 && frame.regions == base.scene.regions,
                    "unchanged image-bearing cells repainted instead of replaying their cache");
                let replay = Box::new(capture(frame, paint, pane.ok_or_else(|| anyhow::anyhow!("replay pane missing"))?, window)?);
                ensure!(replay.scene.serials == base.scene.serials, "stationary native assets changed");
                eprintln!("IMAGE_NATIVE_WHEEL_READY cells=0 regions={} uploads={}", replay.scene.regions.len(), replay.paint.images.len());
                Stage::Wheel { base, replay }
            }
            Stage::Wheel { base, replay } => {
                let crossing = pane.is_some_and(|pane| paint.images.iter().any(|image|
                    image.z == 4 && image.uploaded
                        && f32::from(image.visible.bottom() - pane.bottom()) > 1. / base.scale));
                if matches!(observer.wheel_frames, 0 | 3 | 15 | 60) {
                    eprintln!("IMAGE_NATIVE_WAIT moving={} offset={:?} crossing={crossing} coherent={}",
                        frame.moving, frame.slide.as_ref().map(|slide| slide.offset), frame.serials.is_some());
                }
                observer.wheel_frames += 1;
                if let Some(offset) = frame.slide.as_ref().filter(|slide| frame.moving && crossing && slide.offset.fract().abs() > 0.001).map(|slide| slide.offset) {
                    let moving = Box::new(capture(frame, paint, pane.ok_or_else(|| anyhow::anyhow!("moving pane missing"))?, window)?);
                    scene::validate_motion(&base, &moving, offset)?;
                    eprintln!("IMAGE_NATIVE_FRACTIONAL offset={offset} uploads={}", moving.paint.images.len());
                    Stage::Settle { base, replay, moving }
                } else { Stage::Wheel { base, replay } }
            }
            Stage::Settle { base, replay, moving } => {
                if !frame.moving {
                    let settled = Box::new(capture(frame, paint, pane.ok_or_else(|| anyhow::anyhow!("settled pane missing"))?, window)?);
                    ensure!(settled.scene.serials == base.scene.serials, "settled native assets changed");
                    ensure!(settled.scene.frame.panes[0].scroll.map(|s| s.offset_from_bottom)
                        != base.scene.frame.panes[0].scroll.map(|s| s.offset_from_bottom),
                        "native wheel did not change authoritative daemon scrollback");
                    eprintln!("IMAGE_NATIVE_CAPTURED stationary=true replay=true fractional=true settled=true");
                    observer.stage = Stage::Encoding;
                    return Ok(Some([base, replay, moving, settled]));
                }
                Stage::Settle { base, replay, moving }
            }
            stage => stage,
        };
        Ok(None)
    })?;
    if let Some(shots) = completed {
        cx.spawn(async move |cx| {
            if let Err(error) = receipts(shots, cx).await { fail(error); }
            OBSERVER.with_borrow_mut(|slot| {
                if let Some(observer) = slot { observer.stage = Stage::Done; }
            });
            eprintln!("IMAGE_NATIVE_PASS assets=true cached_cells=true fractional=true clipping=true layers=true");
        }).detach();
    } else {
        entity.update(cx, |_, cx| cx.notify());
    }
    Ok(())
}

async fn receipts(shots: [Box<Shot>; 4], cx: &AsyncApp) -> Result<()> {
    for (name, shot) in ["stationary", "replay", "fractional", "settled"]
        .into_iter()
        .zip(shots)
    {
        receipt(name, shot.pixels, cx).await?;
    }
    Ok(())
}

async fn receipt(name: &str, pixels: image::RgbaImage, cx: &AsyncApp) -> Result<()> {
    let dimensions = pixels.dimensions();
    ensure!(
        u64::from(dimensions.0) * u64::from(dimensions.1) <= 2_000_000,
        "native image capture exceeds pixel bound"
    );
    let png = cx
        .background_spawn(async move {
            use image::codecs::png::{CompressionType, FilterType, PngEncoder};
            let mut png = Vec::new();
            pixels.write_with_encoder(PngEncoder::new_with_quality(
                &mut png,
                CompressionType::Best,
                FilterType::Adaptive,
            ))?;
            ensure!(png.len() <= 131_072, "native image PNG exceeds 128 KiB");
            anyhow::Ok(png)
        })
        .await?;
    eprintln!(
        "IMAGE_NATIVE_PNG {name} {}",
        base64::engine::general_purpose::STANDARD.encode(&png)
    );
    eprintln!(
        "IMAGE_NATIVE_FRAME {name} dimensions={dimensions:?} bytes={}",
        png.len()
    );
    Ok(())
}
