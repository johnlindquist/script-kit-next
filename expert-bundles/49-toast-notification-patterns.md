# Script Kit GPUI - Expert Bundle 49: Toast/Notification Patterns

## Project Context

Script Kit GPUI is a **Rust desktop app** built with GPUI (Zed's UI framework) that serves as a command launcher and script runner.

**Notification Types:**
- Toast notifications (in-app, temporary)
- HUD overlays (centered, auto-dismiss)
- System notifications (macOS Notification Center)
- Error dialogs (blocking, require action)

---

## Goal

Create a **unified notification system** that:
1. Provides consistent toast/notification UI
2. Supports multiple notification channels
3. Handles notification queuing and stacking
4. Enables scripts to trigger notifications
5. Integrates with macOS system notifications

---

## Current State

### Notification Sources

| Type | Location | Behavior |
|------|----------|----------|
| Toast | `toast_manager.rs` | Queue + gpui-component Notification |
| HUD | `hud_manager.rs` | Centered overlay, auto-dismiss |
| Error | `error.rs` | Toast with error styling |
| Script HUD | Protocol `showHud` | HUD overlay |

### Current Implementation

```rust
// toast_manager.rs
pub struct ToastManager {
    pending_toasts: VecDeque<PendingToast>,
}

pub struct PendingToast {
    pub message: String,
    pub variant: ToastVariant,
    pub duration_ms: Option<u64>,
}

pub enum ToastVariant {
    Success,
    Warning,
    Error,
    Info,
}

// Usage: Push toast then flush in render()
self.toast_manager.push(PendingToast {
    message: "Script completed".to_string(),
    variant: ToastVariant::Success,
    duration_ms: Some(3000),
});
```

### Problems

1. **Two Systems** - Toast and HUD are separate
2. **No Stacking** - Multiple toasts don't stack nicely
3. **No Actions** - Can't have buttons on toasts
4. **No Persistence** - Important notifications lost on dismiss
5. **SDK Limited** - Only HUD available to scripts
6. **No Grouping** - Can't collapse similar notifications

---

## Proposed Architecture

### 1. Unified Notification Model

```rust
/// A notification that can be displayed in various ways
#[derive(Clone, Debug)]
pub struct Notification {
    /// Unique identifier
    pub id: NotificationId,
    /// Notification content
    pub content: NotificationContent,
    /// How to display the notification
    pub style: NotificationStyle,
    /// Behavior configuration
    pub behavior: NotificationBehavior,
    /// Optional actions
    pub actions: Vec<NotificationAction>,
    /// Source for grouping
    pub source: NotificationSource,
    /// Creation timestamp
    pub created_at: Instant,
}

pub type NotificationId = u64;

#[derive(Clone, Debug)]
pub enum NotificationContent {
    /// Simple text message
    Text(String),
    /// Title + message
    TitleMessage { title: String, message: String },
    /// Rich content with icon
    Rich {
        icon: Option<IconRef>,
        title: String,
        message: Option<String>,
    },
    /// Progress indicator
    Progress {
        title: String,
        progress: f32,  // 0.0 - 1.0
        message: Option<String>,
    },
    /// Custom HTML (for HUD)
    Html(String),
}

#[derive(Clone, Copy, Debug, Default)]
pub enum NotificationStyle {
    /// Small toast in corner
    #[default]
    Toast,
    /// Centered HUD overlay
    Hud,
    /// Banner at top of window
    Banner,
    /// Inline in current view
    Inline,
    /// System notification (macOS)
    System,
}

#[derive(Clone, Debug)]
pub struct NotificationBehavior {
    /// Auto-dismiss after duration (None = persistent)
    pub duration: Option<Duration>,
    /// Can user dismiss manually
    pub dismissable: bool,
    /// Replace previous notification with same source
    pub replace_existing: bool,
    /// Sound to play
    pub sound: Option<NotificationSound>,
    /// Priority for ordering
    pub priority: NotificationPriority,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum NotificationPriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,  // Plays sound, doesn't auto-dismiss
}

#[derive(Clone, Copy, Debug)]
pub enum NotificationSound {
    Default,
    Success,
    Error,
    Custom(&'static str),
}

#[derive(Clone, Debug)]
pub struct NotificationAction {
    pub label: String,
    pub id: String,
    pub style: ActionStyle,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum ActionStyle {
    #[default]
    Default,
    Primary,
    Destructive,
}

#[derive(Clone, Debug)]
pub enum NotificationSource {
    System,
    Script { path: String },
    BuiltIn { id: String },
}
```

### 2. Notification Service

```rust
/// Centralized notification management
pub struct NotificationService {
    /// Active notifications
    notifications: Vec<Notification>,
    /// Notification history (for review)
    history: VecDeque<Notification>,
    /// Max history size
    max_history: usize,
    /// Listeners for notification events
    listeners: Vec<Box<dyn NotificationListener>>,
    /// ID counter
    next_id: NotificationId,
}

pub trait NotificationListener: Send + Sync {
    fn on_notification(&self, notification: &Notification);
    fn on_dismiss(&self, id: NotificationId, reason: DismissReason);
    fn on_action(&self, id: NotificationId, action_id: &str);
}

#[derive(Clone, Copy, Debug)]
pub enum DismissReason {
    Timeout,
    UserDismissed,
    Replaced,
    ActionTaken,
    Cleared,
}

impl NotificationService {
    pub fn global() -> &'static Mutex<NotificationService> {
        static SERVICE: OnceLock<Mutex<NotificationService>> = OnceLock::new();
        SERVICE.get_or_init(|| Mutex::new(NotificationService::new()))
    }
    
    /// Show a notification
    pub fn notify(&mut self, notification: Notification, cx: &mut App) {
        let id = notification.id;
        
        // Handle replacement
        if notification.behavior.replace_existing {
            self.notifications.retain(|n| {
                if matches_source(&n.source, &notification.source) {
                    self.notify_listeners_dismiss(n.id, DismissReason::Replaced);
                    false
                } else {
                    true
                }
            });
        }
        
        // Play sound
        if let Some(sound) = &notification.behavior.sound {
            play_notification_sound(*sound);
        }
        
        // Route to appropriate renderer
        match notification.style {
            NotificationStyle::Toast => self.show_toast(&notification, cx),
            NotificationStyle::Hud => self.show_hud(&notification, cx),
            NotificationStyle::Banner => self.show_banner(&notification, cx),
            NotificationStyle::System => self.show_system(&notification),
            NotificationStyle::Inline => {} // Handled by caller
        }
        
        // Store notification
        self.notifications.push(notification.clone());
        
        // Schedule auto-dismiss
        if let Some(duration) = notification.behavior.duration {
            self.schedule_dismiss(id, duration, cx);
        }
        
        // Notify listeners
        for listener in &self.listeners {
            listener.on_notification(&notification);
        }
    }
    
    /// Convenience methods
    pub fn success(&mut self, message: impl Into<String>, cx: &mut App) {
        self.notify(Notification::success(message), cx);
    }
    
    pub fn error(&mut self, message: impl Into<String>, cx: &mut App) {
        self.notify(Notification::error(message), cx);
    }
    
    pub fn warning(&mut self, message: impl Into<String>, cx: &mut App) {
        self.notify(Notification::warning(message), cx);
    }
    
    pub fn info(&mut self, message: impl Into<String>, cx: &mut App) {
        self.notify(Notification::info(message), cx);
    }
    
    pub fn progress(&mut self, title: impl Into<String>, progress: f32, cx: &mut App) -> NotificationId {
        let notification = Notification::progress(title, progress);
        let id = notification.id;
        self.notify(notification, cx);
        id
    }
    
    /// Update progress notification
    pub fn update_progress(&mut self, id: NotificationId, progress: f32, cx: &mut App) {
        if let Some(n) = self.notifications.iter_mut().find(|n| n.id == id) {
            if let NotificationContent::Progress { progress: p, .. } = &mut n.content {
                *p = progress;
                cx.notify();
            }
        }
    }
    
    /// Dismiss notification
    pub fn dismiss(&mut self, id: NotificationId, cx: &mut App) {
        if let Some(pos) = self.notifications.iter().position(|n| n.id == id) {
            let notification = self.notifications.remove(pos);
            self.add_to_history(notification);
            self.notify_listeners_dismiss(id, DismissReason::UserDismissed);
            cx.notify();
        }
    }
    
    /// Get notification history
    pub fn history(&self) -> &VecDeque<Notification> {
        &self.history
    }
    
    /// Clear all notifications
    pub fn clear_all(&mut self, cx: &mut App) {
        for n in &self.notifications {
            self.notify_listeners_dismiss(n.id, DismissReason::Cleared);
        }
        self.notifications.clear();
        cx.notify();
    }
}
```

### 3. Notification Builders

```rust
impl Notification {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self {
            id: COUNTER.fetch_add(1, Ordering::SeqCst),
            content: NotificationContent::Text(String::new()),
            style: NotificationStyle::Toast,
            behavior: NotificationBehavior::default(),
            actions: Vec::new(),
            source: NotificationSource::System,
            created_at: Instant::now(),
        }
    }
    
    pub fn success(message: impl Into<String>) -> Self {
        Self::new()
            .content(NotificationContent::Rich {
                icon: Some(IconRef::Lucide(LucideIcon::CheckCircle)),
                title: message.into(),
                message: None,
            })
            .duration(Duration::from_secs(3))
    }
    
    pub fn error(message: impl Into<String>) -> Self {
        Self::new()
            .content(NotificationContent::Rich {
                icon: Some(IconRef::Lucide(LucideIcon::XCircle)),
                title: "Error".to_string(),
                message: Some(message.into()),
            })
            .duration(Duration::from_secs(5))
            .priority(NotificationPriority::High)
    }
    
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new()
            .content(NotificationContent::Rich {
                icon: Some(IconRef::Lucide(LucideIcon::AlertTriangle)),
                title: message.into(),
                message: None,
            })
            .duration(Duration::from_secs(4))
    }
    
    pub fn info(message: impl Into<String>) -> Self {
        Self::new()
            .content(NotificationContent::Rich {
                icon: Some(IconRef::Lucide(LucideIcon::Info)),
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
            .behavior(NotificationBehavior {
                duration: None,  // Persistent until complete
                dismissable: true,
                replace_existing: true,
                ..Default::default()
            })
    }
    
    pub fn hud(html: impl Into<String>) -> Self {
        Self::new()
            .style(NotificationStyle::Hud)
            .content(NotificationContent::Html(html.into()))
            .duration(Duration::from_secs(2))
    }
    
    // Builder methods
    pub fn content(mut self, c: NotificationContent) -> Self { self.content = c; self }
    pub fn style(mut self, s: NotificationStyle) -> Self { self.style = s; self }
    pub fn duration(mut self, d: Duration) -> Self { 
        self.behavior.duration = Some(d); 
        self 
    }
    pub fn persistent(mut self) -> Self { 
        self.behavior.duration = None; 
        self 
    }
    pub fn priority(mut self, p: NotificationPriority) -> Self { 
        self.behavior.priority = p; 
        self 
    }
    pub fn action(mut self, label: impl Into<String>, id: impl Into<String>) -> Self {
        self.actions.push(NotificationAction {
            label: label.into(),
            id: id.into(),
            style: ActionStyle::Default,
        });
        self
    }
    pub fn primary_action(mut self, label: impl Into<String>, id: impl Into<String>) -> Self {
        self.actions.push(NotificationAction {
            label: label.into(),
            id: id.into(),
            style: ActionStyle::Primary,
        });
        self
    }
    pub fn source(mut self, s: NotificationSource) -> Self { self.source = s; self }
}
```

### 4. SDK Integration

```typescript
// TypeScript SDK
interface NotifyOptions {
  title?: string;
  message?: string;
  icon?: string;
  style?: 'toast' | 'hud' | 'banner' | 'system';
  duration?: number;  // ms, 0 = persistent
  actions?: Array<{
    label: string;
    value: any;
  }>;
  sound?: 'default' | 'success' | 'error' | boolean;
  replace?: boolean;  // Replace previous from same script
}

// Simple notifications
await notify("Task completed!");
await notify({ title: "Success", message: "File saved" });

// With options
await notify({
  title: "Build Complete",
  message: "42 files compiled",
  icon: "lucide:check-circle",
  style: "toast",
  duration: 5000,
  sound: "success",
});

// With actions (returns selected action value)
const action = await notify({
  title: "Update Available",
  message: "Version 2.0 is ready",
  actions: [
    { label: "Update Now", value: "update" },
    { label: "Later", value: "dismiss" },
  ],
});

if (action === "update") {
  // Handle update
}

// Progress notification
const progress = await notify.progress("Downloading...");
for (let i = 0; i <= 100; i += 10) {
  await progress.update(i / 100, `${i}% complete`);
  await sleep(100);
}
await progress.complete("Download complete!");

// HUD (centered overlay)
await notify.hud("Copied to clipboard!");
await notify.hud(`<div class="text-2xl">🎉</div>`);

// System notification (macOS Notification Center)
await notify.system({
  title: "Reminder",
  message: "Meeting in 5 minutes",
  sound: true,
});
```

---

## Implementation Checklist

### Phase 1: Core Model
- [ ] Define `Notification` and related types
- [ ] Create `NotificationService`
- [ ] Implement notification lifecycle

### Phase 2: Renderers
- [ ] Toast renderer (corner stack)
- [ ] HUD renderer (centered overlay)
- [ ] Banner renderer (top of window)
- [ ] System notification (macOS)

### Phase 3: Features
- [ ] Action buttons on notifications
- [ ] Progress notifications
- [ ] Notification grouping
- [ ] Sound effects

### Phase 4: SDK Integration
- [ ] Add notify to protocol
- [ ] TypeScript SDK functions
- [ ] Action callbacks

### Phase 5: History
- [ ] Notification history storage
- [ ] History view built-in
- [ ] Clear history

---

## Key Questions

1. How many toasts should stack before collapsing?
2. Should notifications persist across window hide/show?
3. How to handle script notifications when window is hidden?
4. Should system notifications deep-link back to Script Kit?
5. Should there be a "Do Not Disturb" mode?

---

## Related Bundles

- Bundle 17: HUD Notifications - current HUD implementation
- Bundle 42: App Shell - toast rendering location
- Bundle 32: System Events - system notification integration
