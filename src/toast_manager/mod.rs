//! Toast Manager for coordinating toast notifications
//!
//! This module provides a `ToastManager` that handles:
//! - Notification queue with auto-dismiss timers
//! - Maximum visible toasts limit
//! - Toast positioning (top-right stack)
//! - Dismiss callbacks and lifecycle management
//!
//! # Integration with gpui-component
//!
//! The ToastManager acts as a staging queue. Toasts are pushed via `push()` from
//! anywhere in the code (even without window access). Then in the render loop,
//! call `drain_pending()` to get the pending toasts and push them to gpui-component's
//! notification system via `window.push_notification()`.
//!

#![allow(dead_code)]

include!("part_000.rs");
include!("part_001.rs");
