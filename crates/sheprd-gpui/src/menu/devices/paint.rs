// ABOUTME: Captures the Devices picker and optional session form before owner reset.
// ABOUTME: Keeps scans, accepted mutations, edit targets, and input ownership in live state.
use super::super::sessions::EditPaint;
use super::*;
use picker::PickerPaint;

pub(in crate::menu) struct DevicesPaint {
    picker: PickerPaint,
    session: Option<EditPaint>,
}

impl DevicesPaint {
    pub(in crate::menu) fn session_modal(&self) -> bool {
        self.session.is_some()
    }
}

impl SheprdWindow {
    pub(in crate::menu) fn devices_paint(&self, cx: &App) -> Option<DevicesPaint> {
        (self.menu.page == Some(Page::Devices)).then(|| DevicesPaint {
            picker: self.devices_picker_paint(cx),
            session: self.session_edit_paint(cx),
        })
    }

    /// The footer picker remains separate from the dialog laid over it.
    pub(in crate::menu) fn render_devices_paint(
        &self,
        paint: &DevicesPaint,
        cx: &mut Context<Self>,
    ) -> Div {
        self.render_devices_picker_paint(&paint.picker, cx)
    }

    pub(in crate::menu) fn render_devices_session_paint(
        &self,
        paint: &DevicesPaint,
        cx: &mut Context<Self>,
    ) -> Option<Div> {
        paint
            .session
            .as_ref()
            .map(|session| self.render_session_edit_paint(session, cx))
    }
}
