pub fn has_accessibility_permission() -> bool {
    crate::selected_text::has_accessibility_permission()
}

pub fn request_accessibility_permission() -> bool {
    crate::selected_text::request_accessibility_permission()
}

pub fn open_accessibility_settings() -> anyhow::Result<()> {
    crate::selected_text::open_accessibility_settings()
}

pub fn show_permission_dialog() -> anyhow::Result<bool> {
    crate::selected_text::show_permission_dialog()
}
