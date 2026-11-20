// ABOUTME: Prepares Core setup text and fields without retaining a request or reply owner.
// ABOUTME: Captures the exact pending status and numeric body position before reset.
use super::*;

pub(super) enum Field {
    // keep-sorted start
    Input(Box<crate::search_input::Paint>),
    Text(String),
    // keep-sorted end
}

pub(in crate::menu::devices) struct Paint {
    pub(super) fields: [Field; 3],
    pub(super) rail: super::super::rail::Paint,
    pub(super) error: Option<String>,
    pub(super) status: Option<String>,
    pub(super) ready: bool,
    pub(super) offset: Point<Pixels>,
}

impl SheprdWindow {
    pub(in crate::menu::devices) fn add_device_paint(&self, cx: &App) -> Option<Paint> {
        let setup = self.menu.device_setup.as_ref()?;
        let pending = matches!(
            setup.step,
            Step::CoreConnecting(_)
                | Step::CoreAdding { .. }
                | Step::CoreWaiting(_)
                | Step::CoreUnknown
        );
        Some(Paint {
            fields: std::array::from_fn(|index| {
                let field = setup.fields[index].read(cx);
                if pending {
                    Field::Text(sheprd_client::text::safe_text(field.text(), 160))
                } else {
                    Field::Input(Box::new(field.paint()))
                }
            }),
            rail: self.device_rail_paint(super::super::rail::Source::Host),
            error: setup.error.clone(),
            status: match &setup.step {
                Step::Form | Step::CoreUnknown | Step::Saved => None,
                Step::CoreConnecting(request) => Some(fl!(
                    "add-device-core-connecting",
                    target = sheprd_client::text::safe_text(&request.target, 160)
                )),
                Step::CoreAdding { request, .. } => Some(fl!(
                    "add-device-core-adding",
                    target = sheprd_client::text::safe_text(&request.target, 160)
                )),
                Step::CoreWaiting(request) => Some(fl!(
                    "add-device-core-waiting",
                    target = sheprd_client::text::safe_text(&request.target, 160)
                )),
            },
            ready: matches!(setup.step, Step::Form),
            offset: setup.scroll.offset(),
        })
    }
}
