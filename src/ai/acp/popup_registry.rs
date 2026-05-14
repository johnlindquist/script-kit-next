use gpui::AnyWindowHandle;

pub(crate) struct AcpPopupRegistration {
    id: &'static str,
}

impl AcpPopupRegistration {
    pub(crate) fn register(id: &'static str, handle: AnyWindowHandle) -> Self {
        crate::windows::upsert_runtime_window_handle(id, handle);
        Self { id }
    }

    pub(crate) fn remove(id: &'static str) {
        crate::windows::remove_runtime_window_handle(id);
        crate::windows::remove_automation_window(id);
    }
}

impl Drop for AcpPopupRegistration {
    fn drop(&mut self) {
        Self::remove(self.id);
    }
}
