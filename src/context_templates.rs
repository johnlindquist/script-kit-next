use std::borrow::Cow;

pub(crate) struct ContextTemplateVars {
    pub app_name: String,
    pub window_title: String,
}

impl ContextTemplateVars {
    pub(crate) fn from_frontmost_tracker() -> Self {
        #[cfg(target_os = "macos")]
        {
            let tracked = crate::frontmost_app_tracker::get_last_real_app();
            return Self {
                app_name: tracked
                    .as_ref()
                    .map(|a| a.name.clone())
                    .unwrap_or_else(|| "Current App".to_string()),
                window_title: tracked
                    .as_ref()
                    .and_then(|a| a.window_title.clone())
                    .unwrap_or_else(|| "Focused Window".to_string()),
            };
        }
        #[cfg(not(target_os = "macos"))]
        {
            Self {
                app_name: "Current App".to_string(),
                window_title: "Focused Window".to_string(),
            }
        }
    }
}

pub(crate) fn substitute_context_vars<'a>(
    text: &'a str,
    vars: &ContextTemplateVars,
) -> Cow<'a, str> {
    if !text.contains("{{") {
        return Cow::Borrowed(text);
    }
    Cow::Owned(
        text.replace("{{app}}", &vars.app_name)
            .replace("{{window}}", &vars.window_title),
    )
}
