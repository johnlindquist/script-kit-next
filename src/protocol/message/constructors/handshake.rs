use super::*;

impl Message {
    // ============================================================
    // PROTOCOL HANDSHAKE CONSTRUCTORS
    // ============================================================

    /// Create a Hello handshake message (SDK → App)
    ///
    /// # Arguments
    /// * `protocol` - Protocol version number (typically 1)
    /// * `sdk_version` - SDK version string (e.g., "1.0.0")
    /// * `capabilities` - List of capability flags the SDK supports
    pub fn hello(protocol: u32, sdk_version: impl Into<String>, capabilities: Vec<String>) -> Self {
        Message::Hello {
            protocol,
            sdk_version: sdk_version.into(),
            capabilities,
        }
    }

    /// Create a HelloAck response message (App → SDK)
    ///
    /// # Arguments
    /// * `protocol` - Protocol version number the app supports
    /// * `capabilities` - List of capability flags the app confirms it supports
    pub fn hello_ack(protocol: u32, capabilities: Vec<String>) -> Self {
        Message::HelloAck {
            protocol,
            capabilities,
        }
    }

    /// Create a HelloAck with all current capabilities enabled
    pub fn hello_ack_full(protocol: u32) -> Self {
        Message::HelloAck {
            protocol,
            capabilities: vec![
                capabilities::SUBMIT_JSON.to_string(),
                capabilities::SEMANTIC_ID_V2.to_string(),
                capabilities::UNKNOWN_TYPE_OK.to_string(),
                capabilities::FORWARD_COMPAT.to_string(),
                capabilities::CHOICE_KEY.to_string(),
                capabilities::MOUSE_DATA_V2.to_string(),
            ],
        }
    }
}
