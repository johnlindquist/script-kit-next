//! TypeScript config file editor
//!
//! Provides robust utilities for programmatically modifying TypeScript config files
//! like `~/.scriptkit/kit/config.ts`.
//!
//! # Design
//!
//! Uses tree-sitter-typescript to parse the AST, giving exact byte offsets for
//! insertion points. This eliminates the fragility of hand-rolled brace counting.

include!("editor/part_01.rs");
include!("editor/part_02.rs");

#[cfg(test)]
#[path = "editor/tests.rs"]
mod tests;
