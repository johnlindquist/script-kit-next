//! Zero-copy webcam capture via AVFoundation.
//!
//! Captures camera frames as CVPixelBuffer and sends them directly to
//! GPUI's `surface()` element — no CPU pixel conversion, no copies.
//!
//! The returned [`CaptureHandle`] owns all AVFoundation resources and
//! stops capture + releases everything on drop.

include!("part_000.rs");
include!("part_001.rs");
