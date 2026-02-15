//! Input detection module for smart fallback commands
//!
//! This module provides functions to detect the type of user input
//! for displaying relevant fallback commands (Raycast-style).

mod detection;

pub use detection::{
    detect_input_type, is_directory_path, is_file_path, is_math_expression, is_url, InputType,
};

#[cfg(test)]
pub(crate) use detection::is_code_snippet;

#[cfg(test)]
mod tests;
