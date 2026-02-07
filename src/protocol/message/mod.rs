//! Protocol Message enum for Script Kit GPUI
//!
//! This module contains the main Message enum that represents all possible
//! protocol messages exchanged between scripts and the GPUI app.

use serde::{Deserialize, Serialize};

use super::types::*;

include!("variants/prompts_media.rs");
include!("variants/system_control.rs");
include!("variants/query_ops.rs");
include!("variants/ai.rs");

macro_rules! protocol_message_define_enum {
    ($($variants:tt)*) => {
        /// Protocol message with type discrimination via serde tag
        ///
        /// This enum uses the "type" field to discriminate between message kinds.
        /// Each variant corresponds to a message kind in the Script Kit v1 API.
        #[derive(Clone, Debug, Serialize, Deserialize)]
        #[serde(tag = "type")]
        #[allow(clippy::large_enum_variant)]
        #[allow(clippy::enum_variant_names)]
        pub enum Message {
            $($variants)*
        }
    };
}

macro_rules! protocol_message_append_ai {
    ($($variants:tt)*) => {
        protocol_message_variants_ai!(protocol_message_define_enum, $($variants)*);
    };
}

macro_rules! protocol_message_append_query_ops {
    ($($variants:tt)*) => {
        protocol_message_variants_query_ops!(protocol_message_append_ai, $($variants)*);
    };
}

macro_rules! protocol_message_append_system_control {
    ($($variants:tt)*) => {
        protocol_message_variants_system_control!(
            protocol_message_append_query_ops,
            $($variants)*
        );
    };
}

protocol_message_variants_prompts_media!(protocol_message_append_system_control);

/// Known protocol capability flags
///
/// These constants represent the capability flags that can be exchanged
/// during the Hello/HelloAck handshake.
pub mod capabilities {
    /// Submit values can be JSON (arrays, objects) not just strings
    pub const SUBMIT_JSON: &str = "submitJson";
    /// Semantic IDs use key-based format when key field is present
    pub const SEMANTIC_ID_V2: &str = "semanticIdV2";
    /// Unknown message types are gracefully handled (not errors)
    pub const UNKNOWN_TYPE_OK: &str = "unknownTypeOk";
    /// Forward-compatibility: extra fields preserved via flatten
    pub const FORWARD_COMPAT: &str = "forwardCompat";
    /// Stable Choice.key field for deterministic IDs
    pub const CHOICE_KEY: &str = "choiceKey";
    /// MouseData struct instead of untagged enum
    pub const MOUSE_DATA_V2: &str = "mouseDataV2";
}

#[path = "constructors/final_sections.rs"]
mod constructors_final_sections;
#[path = "constructors/general.rs"]
mod constructors_general;
#[path = "constructors/handshake.rs"]
mod constructors_handshake;
#[path = "constructors/history_window.rs"]
mod constructors_history_window;
#[path = "constructors/prompts.rs"]
mod constructors_prompts;
#[path = "constructors/query_ops.rs"]
mod constructors_query_ops;

#[cfg(test)]
mod tests;
