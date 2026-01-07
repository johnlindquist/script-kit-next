//! Unified Notification System
//!
//! Centralized notification system supporting multiple delivery channels
//! (toast, HUD, system notifications, dialogs) with consistent behavior.
//!
//! # Usage
//!
//! Initialize the service at app startup:
//! ```rust,ignore
//! NotificationService::init(cx);
//! ```
//!
//! Then use it via update_global:
//! ```rust,ignore
//! cx.update_global::<NotificationService, _>(|service, cx| {
//!     service.success("Task completed!", cx);
//! });
//! ```

mod service;
mod types;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod service_tests;

pub use service::NotificationService;
pub use types::*;
