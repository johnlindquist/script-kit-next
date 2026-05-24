//! Low-level AX access belongs here.
//!
//! The initial inline-agent contracts keep raw AX handles out of DTOs. Native
//! handle storage will live in a short-lived process-local registry owned by
//! this module.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AxSessionHandle {
    pub(crate) session_id: super::FocusedTextSessionId,
}
