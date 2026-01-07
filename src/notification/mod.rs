//! Unified Notification System
//!
//! Centralized notification system supporting multiple delivery channels
//! (toast, HUD, system notifications, dialogs) with consistent behavior.

mod types;

#[cfg(test)]
mod tests;

pub use types::*;
