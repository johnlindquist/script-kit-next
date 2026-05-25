//! Compiles user prompt text, context parts, and system instructions into provider-ready messages.

mod model;
#[cfg(test)]
mod tests;

pub(crate) use model::{
    PromptCompilerContext, PromptCompilerDecision, PromptCompilerError, PromptCompilerPreview,
    PromptCompilerRow, PromptCompilerRowKind, PromptCompilerSnapshot,
};
