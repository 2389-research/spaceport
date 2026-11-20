// ABOUTME: Prepares a host removal confirmation without an external CLI target.
// ABOUTME: Lets native checks detect a held Enter crossing the consent boundary.
use super::*;

impl SheprdWindow {
    pub(crate) fn prepare_host_confirmation_check(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.open_menu(window, cx) {
            return false;
        }
        self.menu.host = Some(HostMenu {
            id: "native-confirmation-fixture".into(),
            saved: Saved::Local,
            label: "Confirmation fixture".into(), // i18n: data
            target: String::new(),
            session: String::new(),
            selected: None,
            input: None,
            renaming: false,
            error: None,
            theme: None,
            tags: None,
            scroll: ScrollHandle::new(),
        });
        self.menu.page = Some(Page::RemoveDevice);
        cx.notify();
        true
    }

    pub(crate) fn host_confirmation_untouched(&self) -> bool {
        self.menu.page == Some(Page::RemoveDevice)
            && self
                .menu
                .host
                .as_ref()
                .is_some_and(|host| host.id == "native-confirmation-fixture")
            && !self
                .menu
                .removing_devices
                .contains("native-confirmation-fixture")
    }
}
