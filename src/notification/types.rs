//! Core notification types

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::icons::IconRef;

/// Unique identifier for a notification
pub type NotificationId = u64;

static NOTIFICATION_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a new unique notification ID
pub fn next_notification_id() -> NotificationId {
    NOTIFICATION_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Delivery channel for a notification
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NotificationChannel {
    InAppToast,
    InAppHud,
    InAppBanner,
    InAppInline,
    System,
    Dialog,
}

/// Content to display in a notification
#[derive(Clone, Debug)]
pub enum NotificationContent {
    Text(String),
    TitleMessage {
        title: String,
        message: String,
    },
    Rich {
        icon: Option<IconRef>,
        title: String,
        message: Option<String>,
    },
    Progress {
        title: String,
        progress: f32,
        message: Option<String>,
    },
    Html(String),
}

impl Default for NotificationContent {
    fn default() -> Self {
        NotificationContent::Text(String::new())
    }
}

/// Sound to play when notification appears
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NotificationSound {
    Default,
    Success,
    Error,
    #[default]
    None,
}

/// Priority level for ordering and routing
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum NotificationPriority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
    Urgent = 3,
}

/// Behavior configuration for a notification
#[derive(Clone, Debug)]
pub struct NotificationBehavior {
    pub duration: Option<Duration>,
    pub dismissable: bool,
    pub replace_key: Option<String>,
    pub sound: NotificationSound,
    pub priority: NotificationPriority,
}

impl Default for NotificationBehavior {
    fn default() -> Self {
        Self {
            duration: Some(Duration::from_secs(3)),
            dismissable: true,
            replace_key: None,
            sound: NotificationSound::None,
            priority: NotificationPriority::Normal,
        }
    }
}

/// Visual style for an action button
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ActionStyle {
    #[default]
    Default,
    Primary,
    Destructive,
}

/// An action button on a notification
#[derive(Clone, Debug)]
pub struct NotificationAction {
    pub label: String,
    pub id: String,
    pub style: ActionStyle,
}

impl NotificationAction {
    pub fn new(label: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id: id.into(),
            style: ActionStyle::Default,
        }
    }

    pub fn primary(label: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id: id.into(),
            style: ActionStyle::Primary,
        }
    }

    pub fn destructive(label: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id: id.into(),
            style: ActionStyle::Destructive,
        }
    }
}

/// Source of a notification (for grouping and rate limiting)
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum NotificationSource {
    #[default]
    System,
    Script {
        path: String,
    },
    BuiltIn {
        id: String,
    },
}

/// Reason why a notification was dismissed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DismissReason {
    Timeout,
    UserDismissed,
    Replaced,
    ActionTaken,
    Cleared,
    Programmatic,
}

/// A notification that can be displayed through various channels
#[derive(Clone, Debug)]
pub struct Notification {
    pub id: NotificationId,
    pub content: NotificationContent,
    pub channels: Vec<NotificationChannel>,
    pub behavior: NotificationBehavior,
    pub actions: Vec<NotificationAction>,
    pub source: NotificationSource,
    pub group_key: Option<String>,
    pub dedupe_key: Option<String>,
    pub created_at: Instant,
}

impl Default for Notification {
    fn default() -> Self {
        Self::new()
    }
}

impl Notification {
    pub fn new() -> Self {
        Self {
            id: next_notification_id(),
            content: NotificationContent::default(),
            channels: vec![NotificationChannel::InAppToast],
            behavior: NotificationBehavior::default(),
            actions: Vec::new(),
            source: NotificationSource::System,
            group_key: None,
            dedupe_key: None,
            created_at: Instant::now(),
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new()
            .content(NotificationContent::Rich {
                icon: IconRef::parse("lucide:check-circle"),
                title: message.into(),
                message: None,
            })
            .duration(Duration::from_secs(3))
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new()
            .content(NotificationContent::Rich {
                icon: IconRef::parse("lucide:x-circle"),
                title: "Error".to_string(),
                message: Some(message.into()),
            })
            .duration(Duration::from_secs(5))
            .priority(NotificationPriority::High)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new()
            .content(NotificationContent::Rich {
                icon: IconRef::parse("lucide:alert-triangle"),
                title: message.into(),
                message: None,
            })
            .duration(Duration::from_secs(4))
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new()
            .content(NotificationContent::Rich {
                icon: IconRef::parse("lucide:info"),
                title: message.into(),
                message: None,
            })
            .duration(Duration::from_secs(3))
    }

    pub fn progress(title: impl Into<String>, progress: f32) -> Self {
        Self::new()
            .content(NotificationContent::Progress {
                title: title.into(),
                progress,
                message: None,
            })
            .persistent()
            .with_replace_key("progress")
    }

    pub fn hud(message: impl Into<String>) -> Self {
        Self::new()
            .channel(NotificationChannel::InAppHud)
            .content(NotificationContent::Text(message.into()))
            .duration(Duration::from_secs(2))
    }

    pub fn hud_html(html: impl Into<String>) -> Self {
        Self::new()
            .channel(NotificationChannel::InAppHud)
            .content(NotificationContent::Html(html.into()))
            .duration(Duration::from_secs(2))
    }

    // Builder methods
    pub fn content(mut self, content: NotificationContent) -> Self {
        self.content = content;
        self
    }

    pub fn channel(mut self, channel: NotificationChannel) -> Self {
        self.channels = vec![channel];
        self
    }

    pub fn add_channel(mut self, channel: NotificationChannel) -> Self {
        if !self.channels.contains(&channel) {
            self.channels.push(channel);
        }
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.behavior.duration = Some(duration);
        self
    }

    pub fn persistent(mut self) -> Self {
        self.behavior.duration = None;
        self
    }

    pub fn dismissable(mut self, dismissable: bool) -> Self {
        self.behavior.dismissable = dismissable;
        self
    }

    pub fn with_replace_key(mut self, key: impl Into<String>) -> Self {
        self.behavior.replace_key = Some(key.into());
        self
    }

    pub fn sound(mut self, sound: NotificationSound) -> Self {
        self.behavior.sound = sound;
        self
    }

    pub fn priority(mut self, priority: NotificationPriority) -> Self {
        self.behavior.priority = priority;
        self
    }

    pub fn action(mut self, label: impl Into<String>, id: impl Into<String>) -> Self {
        self.actions.push(NotificationAction::new(label, id));
        self
    }

    pub fn primary_action(mut self, label: impl Into<String>, id: impl Into<String>) -> Self {
        self.actions.push(NotificationAction::primary(label, id));
        self
    }

    pub fn source(mut self, source: NotificationSource) -> Self {
        self.source = source;
        self
    }

    pub fn from_script(mut self, path: impl Into<String>) -> Self {
        let p = path.into();
        self.source = NotificationSource::Script { path: p.clone() };
        if self.behavior.replace_key.is_none() {
            self.behavior.replace_key = Some(format!("script:{}", p));
        }
        self
    }

    pub fn group(mut self, key: impl Into<String>) -> Self {
        self.group_key = Some(key.into());
        self
    }

    pub fn dedupe(mut self, key: impl Into<String>) -> Self {
        self.dedupe_key = Some(key.into());
        self
    }

    // Query methods
    pub fn source_key(&self) -> String {
        match &self.source {
            NotificationSource::System => "system".to_string(),
            NotificationSource::Script { path } => path.clone(),
            NotificationSource::BuiltIn { id } => format!("builtin:{}", id),
        }
    }

    pub fn should_auto_dismiss(&self) -> bool {
        self.behavior.duration.is_some()
    }

    pub fn is_expired(&self) -> bool {
        self.behavior
            .duration
            .map(|d| self.created_at.elapsed() >= d)
            .unwrap_or(false)
    }

    pub fn remaining_ms(&self) -> Option<u64> {
        self.behavior.duration.map(|duration| {
            let elapsed = self.created_at.elapsed().as_millis() as u64;
            let total = duration.as_millis() as u64;
            total.saturating_sub(elapsed)
        })
    }
}

/// A notification stored in history
#[derive(Clone, Debug)]
pub struct NotificationHistoryEntry {
    pub notification: Notification,
    pub dismissed_at: Instant,
    pub dismiss_reason: DismissReason,
}

/// An active notification with runtime state
#[derive(Clone, Debug)]
pub struct ActiveNotification {
    pub notification: Notification,
    pub timer_paused: bool,
    pub paused_at: Option<Instant>,
    pub paused_duration: Duration,
    pub dedupe_count: u32,
}

impl ActiveNotification {
    pub fn new(notification: Notification) -> Self {
        Self {
            notification,
            timer_paused: false,
            paused_at: None,
            paused_duration: Duration::ZERO,
            dedupe_count: 0,
        }
    }

    pub fn pause_timer(&mut self) {
        if !self.timer_paused {
            self.timer_paused = true;
            self.paused_at = Some(Instant::now());
        }
    }

    pub fn resume_timer(&mut self) {
        if self.timer_paused {
            if let Some(paused_at) = self.paused_at.take() {
                self.paused_duration += paused_at.elapsed();
            }
            self.timer_paused = false;
        }
    }

    pub fn is_expired(&self) -> bool {
        if self.timer_paused {
            return false;
        }
        if let Some(duration) = self.notification.behavior.duration {
            let total_elapsed = self.notification.created_at.elapsed();
            let active_elapsed = total_elapsed.saturating_sub(self.paused_duration);
            active_elapsed >= duration
        } else {
            false
        }
    }

    pub fn increment_dedupe(&mut self) {
        self.dedupe_count += 1;
    }
}
