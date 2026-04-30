//! Thin GPUI adapter helpers.
//!
//! The core crate deliberately stays UI-framework agnostic. This module provides a
//! GPUI `EventEmitter` marker and a small receiver-backed event source that you can
//! own inside a GPUI entity and poll from your app's normal tick/idle path.

use std::sync::mpsc::{Receiver, TryRecvError};

use gpui::EventEmitter;

use crate::AxEvent;

/// Receiver-backed source for AX observer events.
///
/// Own this inside a GPUI entity, call [`drain_pending`](Self::drain_pending)
/// during your app update/tick, then call `cx.emit(event)` and `cx.notify()` for
/// every drained event.
pub struct AxEventSource {
    receiver: Receiver<AxEvent>,
    latest: Option<AxEvent>,
}

impl AxEventSource {
    pub fn new(receiver: Receiver<AxEvent>) -> Self {
        Self {
            receiver,
            latest: None,
        }
    }

    /// Most recent event drained from the receiver.
    pub fn latest(&self) -> Option<&AxEvent> {
        self.latest.as_ref()
    }

    /// Drain all currently queued events without blocking the GPUI thread.
    pub fn drain_pending(&mut self) -> Vec<AxEvent> {
        let mut events = Vec::new();
        loop {
            match self.receiver.try_recv() {
                Ok(event) => {
                    self.latest = Some(event.clone());
                    events.push(event);
                }
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
        events
    }
}

impl EventEmitter<AxEvent> for AxEventSource {}
