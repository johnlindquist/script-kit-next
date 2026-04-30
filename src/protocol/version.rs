//! Oracle-Session `protocol-builtin-boundary-refactor-plan` PR2:
//! wire-format protocol version envelope.
//!
//! Every JSONL message carries an optional `protocolVersion` field.
//! **Absence is intentional** and means v1 — every legacy message
//! published by older SDKs, scripts, or third-party tools remains
//! valid without change.
//!
//! This module is additive. It does not yet rewire
//! [`crate::protocol::io`] parsing. PR2b will wrap
//! `parse_message_graceful` to read the version before dispatch.
//! Splitting the landing keeps the blast radius small and lets the
//! downstream MCP `protocol-stats` resource ship an envelope-aware
//! counter without waiting for the full parse refactor.

use serde_json::Value;
use thiserror::Error;

/// The version this build produces by default when attaching an
/// envelope to an outbound message.
pub const CURRENT_PROTOCOL_VERSION: u16 = 2;

/// The minimum version this build is willing to parse. Inbound
/// messages tagged below this are rejected loudly.
pub const MIN_PROTOCOL_VERSION: u16 = 1;

/// The wire envelope field name — intentionally camelCase to match
/// every other field on the JSONL surface.
pub const PROTOCOL_VERSION_FIELD: &str = "protocolVersion";

/// Parsed envelope version. Distinct from a bare `u16` so it is
/// awkward to silently construct without going through
/// [`read_wire_version`] or [`ProtocolVersion::current`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProtocolVersion(u16);

impl ProtocolVersion {
    /// The version this binary produces by default.
    pub const fn current() -> Self {
        Self(CURRENT_PROTOCOL_VERSION)
    }

    /// The implicit version assumed when no `protocolVersion` field
    /// is present on the wire — always v1.
    pub const fn default_legacy() -> Self {
        Self(MIN_PROTOCOL_VERSION)
    }

    /// Construct from a raw integer without validation. Intended for
    /// tests and snapshot replay; production parsers must go through
    /// [`read_wire_version`].
    pub const fn from_raw(value: u16) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolVersionError {
    #[error("message is not a JSON object, cannot carry a protocolVersion envelope")]
    NotObject,
    #[error("protocolVersion field is present but not an unsigned integer in [0, {max}]", max = u16::MAX)]
    InvalidType,
    #[error(
        "unsupported protocolVersion {found}; this build supports [{min}, {max}]",
        min = MIN_PROTOCOL_VERSION,
        max = CURRENT_PROTOCOL_VERSION
    )]
    Unsupported { found: u16 },
}

/// Read `protocolVersion` from a parsed JSON value. Absence is fine
/// and yields [`ProtocolVersion::default_legacy`] (v1). A present
/// field must be a non-negative integer that fits in `u16` and falls
/// within `[MIN_PROTOCOL_VERSION, CURRENT_PROTOCOL_VERSION]`.
pub fn read_wire_version(value: &Value) -> Result<ProtocolVersion, ProtocolVersionError> {
    let Some(obj) = value.as_object() else {
        return Err(ProtocolVersionError::NotObject);
    };
    let Some(raw) = obj.get(PROTOCOL_VERSION_FIELD) else {
        return Ok(ProtocolVersion::default_legacy());
    };
    let Some(n) = raw.as_u64() else {
        return Err(ProtocolVersionError::InvalidType);
    };
    if n > u16::MAX as u64 {
        return Err(ProtocolVersionError::InvalidType);
    }
    let found = n as u16;
    if !(MIN_PROTOCOL_VERSION..=CURRENT_PROTOCOL_VERSION).contains(&found) {
        return Err(ProtocolVersionError::Unsupported { found });
    }
    Ok(ProtocolVersion(found))
}

/// Stamp [`CURRENT_PROTOCOL_VERSION`] into an object value in place.
/// A no-op if the value is not an object — outbound serializers that
/// require the envelope should guard on the type first rather than
/// silently drop it.
pub fn attach_current_version(value: &mut Value) -> bool {
    let Some(obj) = value.as_object_mut() else {
        return false;
    };
    obj.insert(
        PROTOCOL_VERSION_FIELD.to_string(),
        Value::from(CURRENT_PROTOCOL_VERSION),
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn absent_field_is_v1() {
        let v = json!({ "type": "arg" });
        assert_eq!(read_wire_version(&v), Ok(ProtocolVersion::default_legacy()));
    }

    #[test]
    fn present_valid_field_is_read() {
        let v = json!({ "type": "arg", "protocolVersion": 2 });
        assert_eq!(read_wire_version(&v), Ok(ProtocolVersion::current()));
    }

    #[test]
    fn non_object_is_rejected() {
        let v = json!("hello");
        assert_eq!(read_wire_version(&v), Err(ProtocolVersionError::NotObject));
    }

    #[test]
    fn non_integer_is_rejected() {
        let v = json!({ "protocolVersion": "two" });
        assert_eq!(
            read_wire_version(&v),
            Err(ProtocolVersionError::InvalidType)
        );
    }

    #[test]
    fn negative_is_rejected() {
        let v = json!({ "protocolVersion": -1 });
        assert_eq!(
            read_wire_version(&v),
            Err(ProtocolVersionError::InvalidType)
        );
    }

    #[test]
    fn overflowing_integer_is_rejected() {
        let v = json!({ "protocolVersion": (u16::MAX as u64) + 1 });
        assert_eq!(
            read_wire_version(&v),
            Err(ProtocolVersionError::InvalidType)
        );
    }

    #[test]
    fn too_old_is_rejected() {
        let v = json!({ "protocolVersion": 0 });
        assert_eq!(
            read_wire_version(&v),
            Err(ProtocolVersionError::Unsupported { found: 0 })
        );
    }

    #[test]
    fn too_new_is_rejected() {
        let v = json!({ "protocolVersion": CURRENT_PROTOCOL_VERSION + 1 });
        assert_eq!(
            read_wire_version(&v),
            Err(ProtocolVersionError::Unsupported {
                found: CURRENT_PROTOCOL_VERSION + 1
            })
        );
    }

    #[test]
    fn attach_stamps_current_version() {
        let mut v = json!({ "type": "arg" });
        assert!(attach_current_version(&mut v));
        assert_eq!(
            v.get(PROTOCOL_VERSION_FIELD).and_then(|x| x.as_u64()),
            Some(CURRENT_PROTOCOL_VERSION as u64)
        );
    }

    #[test]
    fn attach_ignores_non_object() {
        let mut v = json!("bare string");
        assert!(!attach_current_version(&mut v));
    }

    #[test]
    fn attach_overwrites_existing() {
        let mut v = json!({ "protocolVersion": 1 });
        assert!(attach_current_version(&mut v));
        assert_eq!(
            v.get(PROTOCOL_VERSION_FIELD).and_then(|x| x.as_u64()),
            Some(CURRENT_PROTOCOL_VERSION as u64)
        );
    }

    #[test]
    fn current_is_at_or_above_min() {
        assert!(CURRENT_PROTOCOL_VERSION >= MIN_PROTOCOL_VERSION);
    }
}
