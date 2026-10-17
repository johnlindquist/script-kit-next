//! What "Send Window to Agent Chat" sends, and the one-line summary the user
//! sees ("Slack — #general · Screenshot · Selection · Page").
//!
//! The command gathers up to four sources at once: the window screenshot,
//! the app and window title, the text selected in that window, and the page
//! address when the app is a browser. Each source is optional and fails on
//! its own: a missing Screen Recording grant still sends the text sources,
//! and says how to add the screenshot.
//!
//! Text from the window goes through [`sensitive_text_reason`] first. A
//! selection that looks like a secret (keys, tokens, private keys, password
//! assignments) is left out and the summary says so.

/// Most selection characters sent with a window.
pub const MAX_SELECTION_CHARS: usize = 8_000;

/// Why a piece of text from the screen is not sent, if it looks sensitive.
pub fn sensitive_text_reason(text: &str) -> Option<&'static str> {
    const SECRET_MARKERS: &[(&str, &str)] = &[
        ("-----BEGIN", "a private key"),
        ("sk-", "an API key"),
        ("sk_live_", "an API key"),
        ("ghp_", "an access token"),
        ("github_pat_", "an access token"),
        ("xoxb-", "an access token"),
        ("xoxp-", "an access token"),
        ("AKIA", "an access key"),
        ("Bearer ", "an access token"),
    ];
    for (marker, what) in SECRET_MARKERS {
        if let Some(index) = text.find(marker) {
            // `sk-` and friends count only as the start of a long token.
            let rest = &text[index + marker.len()..];
            let token_len = rest
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '-')
                .count();
            if marker.starts_with("-----") || *marker == "Bearer " || token_len >= 16 {
                return Some(what);
            }
        }
    }
    let lower = text.to_lowercase();
    for key in ["password", "passwd", "secret", "api_key", "apikey", "token"] {
        for sep in ["=", ":"] {
            if let Some(index) = lower.find(&format!("{key}{sep}")) {
                let value = lower[index + key.len() + 1..].trim_start();
                if value.chars().next().is_some_and(|ch| !ch.is_whitespace()) {
                    return Some("a password or secret");
                }
            }
        }
    }
    None
}

/// The sources gathered for one window.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowContext {
    pub app_name: Option<String>,
    pub window_title: Option<String>,
    /// Width and height of the screenshot, when one was taken.
    pub screenshot_size: Option<(u32, u32)>,
    /// Why there is no screenshot ("Screen Recording is off", an error).
    pub screenshot_missing: Option<String>,
    pub selection: Option<String>,
    /// Why the selection was left out (it looked like a secret).
    pub selection_withheld: Option<&'static str>,
    pub page_url: Option<String>,
}

impl WindowContext {
    /// Applies the sensitive-text policy and the size cap to a selection.
    pub fn with_selection(mut self, selection: Option<String>) -> Self {
        let Some(text) = selection
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
        else {
            return self;
        };
        if let Some(reason) = sensitive_text_reason(&text) {
            self.selection_withheld = Some(reason);
            return self;
        }
        let capped: String = text.chars().take(MAX_SELECTION_CHARS).collect();
        self.selection = Some(capped);
        self
    }

    /// Names of the sources included, in order.
    pub fn sources(&self) -> Vec<&'static str> {
        let mut sources = Vec::new();
        if self.screenshot_size.is_some() {
            sources.push("Screenshot");
        }
        if self.selection.is_some() {
            sources.push("Selection");
        }
        if self.page_url.is_some() {
            sources.push("Page");
        }
        sources
    }

    /// "Slack — #general · Screenshot · Selection".
    pub fn summary(&self) -> String {
        let place = match (self.app_name.as_deref(), self.window_title.as_deref()) {
            (Some(app), Some(title)) if !title.is_empty() && title != app => {
                format!("{app} \u{2014} {title}")
            }
            (Some(app), _) => app.to_string(),
            (None, Some(title)) => title.to_string(),
            (None, None) => "Window".to_string(),
        };
        let sources = self.sources();
        if sources.is_empty() {
            place
        } else {
            format!("{place} \u{b7} {}", sources.join(" \u{b7} "))
        }
    }

    /// What wasn't included and how to fix it, for the line above the input.
    pub fn notice(&self) -> Option<String> {
        let mut parts = Vec::new();
        if let Some(reason) = &self.screenshot_missing {
            parts.push(format!("No screenshot: {reason}"));
        }
        if let Some(what) = self.selection_withheld {
            parts.push(format!("Selection left out: it looks like {what}"));
        }
        (!parts.is_empty()).then(|| parts.join(". "))
    }

    /// The composer text: the summary line, then the text sources as quoted
    /// context, then the request.
    pub fn composer_text(&self) -> String {
        let mut text = format!("[{}]\n", self.summary());
        if let Some(url) = &self.page_url {
            text.push_str(&format!("Page: {url}\n"));
        }
        if let Some(selection) = &self.selection {
            text.push_str("Selected text:\n\"\"\"\n");
            text.push_str(selection);
            text.push_str("\n\"\"\"\n");
        }
        text.push_str(if self.screenshot_size.is_some() {
            "\nPlease look at this window."
        } else {
            "\nPlease help with this window."
        });
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_are_recognised_and_ordinary_text_is_not() {
        assert_eq!(
            sensitive_text_reason("key sk-abcdefghijklmnopqrstu"),
            Some("an API key")
        );
        assert_eq!(
            sensitive_text_reason("-----BEGIN OPENSSH PRIVATE KEY-----"),
            Some("a private key")
        );
        assert_eq!(
            sensitive_text_reason("password: hunter2"),
            Some("a password or secret")
        );
        assert_eq!(
            sensitive_text_reason("Authorization: Bearer abc"),
            Some("an access token")
        );
        assert_eq!(sensitive_text_reason("the task-list is long"), None);
        assert_eq!(sensitive_text_reason("Ask for the password later"), None);
        assert_eq!(sensitive_text_reason("sk-learn is a library"), None);
    }

    #[test]
    fn the_summary_names_the_window_and_its_sources() {
        let context = WindowContext {
            app_name: Some("Slack".into()),
            window_title: Some("#general".into()),
            screenshot_size: Some((1200, 800)),
            page_url: None,
            ..Default::default()
        }
        .with_selection(Some("  ship it today ".into()));
        assert_eq!(
            context.summary(),
            "Slack \u{2014} #general \u{b7} Screenshot \u{b7} Selection"
        );
        let text = context.composer_text();
        assert!(text.starts_with("[Slack \u{2014} #general \u{b7} Screenshot \u{b7} Selection]\n"));
        assert!(text.contains("\"\"\"\nship it today\n\"\"\""));
        assert_eq!(context.notice(), None);
    }

    #[test]
    fn missing_pieces_are_explained() {
        let context = WindowContext {
            app_name: Some("Safari".into()),
            window_title: Some("Safari".into()),
            screenshot_missing: Some(
                "Screen Recording is off. Turn it on in Set Up Permissions".into(),
            ),
            page_url: Some("https://example.com/a".into()),
            ..Default::default()
        }
        .with_selection(Some("token=abc123".into()));
        assert_eq!(context.summary(), "Safari \u{b7} Page");
        assert!(context.selection.is_none());
        let notice = context.notice().expect("notice");
        assert!(
            notice.contains("No screenshot: Screen Recording is off"),
            "{notice}"
        );
        assert!(notice.contains("Selection left out"), "{notice}");
        assert!(context
            .composer_text()
            .ends_with("Please help with this window."));
    }

    #[test]
    fn long_selections_are_capped() {
        let context =
            WindowContext::default().with_selection(Some("a".repeat(MAX_SELECTION_CHARS + 50)));
        assert_eq!(
            context.selection.map(|s| s.chars().count()),
            Some(MAX_SELECTION_CHARS)
        );
    }
}
