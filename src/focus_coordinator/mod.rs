//! Focus Coordinator - Centralized focus management for Script Kit GPUI
//!
//! This module provides a single control plane for focus management, replacing
//! the scattered `focused_input` + `pending_focus` pattern with a unified system.
//!
//! # Architecture
//!
//! The FocusCoordinator owns:
//! - **pending**: The next focus request to apply (applied once in render, then cleared)
//! - **restore_stack**: Stack of focus states for overlay push/pop semantics
//! - **current_cursor_owner**: Single source of truth for cursor blink ownership
//!
//! # Key Concepts
//!
//! - **FocusTarget**: Where focus should go (MainFilter, ActionsDialog, specific prompts)
//! - **CursorOwner**: Which input gets the blinking cursor (for text input UX)
//! - **FocusRequest**: Complete focus intent (target + cursor owner)
//!
//! # Usage Patterns
//!
//! ```rust,ignore
//! // Request focus to main filter with cursor
//! coordinator.request(FocusRequest::main_filter());
//!
//! // Push overlay (actions dialog) - saves current state for restore
//! coordinator.push_overlay(FocusRequest::actions_dialog());
//!
//! // Pop overlay - restores previous focus state
//! coordinator.pop_overlay();
//!
//! // Apply pending focus (called once per render when appropriate)
//! if let Some(request) = coordinator.take_pending() {
//!     // ... apply focus based on request.target
//! }
//! ```

include!("part_000.rs");
include!("part_001.rs");
