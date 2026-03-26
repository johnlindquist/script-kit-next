//! Hybrid snippet planner — resolves known variables, promotes unknowns to tabstops,
//! and routes to immediate-paste or interactive-template based on the result.

use crate::template_variables::{
    promote_unresolved_variables_to_tabstops, substitute_variables_with_receipt, VariableContext,
};

/// Whether the snippet should be pasted immediately or opened as an interactive template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HybridSnippetPlanKind {
    /// All variables resolved, no tabstops — paste directly.
    ImmediatePaste,
    /// Unresolved variables or explicit tabstops — open interactive editor.
    InteractiveTemplate,
}

/// The output of the hybrid snippet planner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HybridSnippetPlan {
    /// Routing decision.
    pub kind: HybridSnippetPlanKind,
    /// Original content before any resolution.
    pub raw_content: String,
    /// Content after variable substitution (unresolved placeholders still present).
    pub resolved_content: String,
    /// Final template ready for the snippet engine (unresolved names promoted to tabstops).
    pub template: String,
    /// Names that had no value in the context.
    pub unresolved_variables: Vec<String>,
    /// Whether the original content contained author-written tabstops (`$1`, `${1:...}`, etc.).
    pub has_explicit_tabstops: bool,
    /// First tabstop index used for promoted variables.
    pub next_promoted_tabstop_index: usize,
}

impl HybridSnippetPlan {
    /// Whether this plan requires user interaction before pasting.
    pub fn needs_interaction(&self) -> bool {
        matches!(self.kind, HybridSnippetPlanKind::InteractiveTemplate)
    }
}

/// Build a hybrid snippet plan: resolve what we can, promote the rest to tabstops.
pub fn build_hybrid_snippet_plan(content: &str, ctx: &VariableContext) -> HybridSnippetPlan {
    let receipt = substitute_variables_with_receipt(content, ctx);
    let has_explicit_tabstops = contains_explicit_tabstops(&receipt.text);
    let next_promoted_tabstop_index = max_explicit_tabstop_index(&receipt.text).saturating_add(1);

    let template = if receipt.unresolved_names.is_empty() {
        receipt.text.clone()
    } else {
        promote_unresolved_variables_to_tabstops(
            &receipt.text,
            &receipt.unresolved_names,
            next_promoted_tabstop_index,
        )
    };

    let kind = if has_explicit_tabstops || !receipt.unresolved_names.is_empty() {
        HybridSnippetPlanKind::InteractiveTemplate
    } else {
        HybridSnippetPlanKind::ImmediatePaste
    };

    tracing::info!(
        raw_len = content.len(),
        resolved_len = receipt.text.len(),
        unresolved_variables = ?receipt.unresolved_names,
        has_explicit_tabstops,
        next_promoted_tabstop_index,
        kind = ?kind,
        "Built hybrid snippet plan"
    );

    HybridSnippetPlan {
        kind,
        raw_content: content.to_string(),
        resolved_content: receipt.text,
        template,
        unresolved_variables: receipt.unresolved_names,
        has_explicit_tabstops,
        next_promoted_tabstop_index,
    }
}

/// Returns `true` if the content contains author-written VSCode-style tabstops.
///
/// Recognizes `$0`, `$1`, `${1:default}`, `${1|a,b|}` but NOT `${name}` (named variables).
pub fn contains_explicit_tabstops(content: &str) -> bool {
    !collect_explicit_tabstop_indices(content).is_empty()
}

/// Returns the highest explicit tabstop index found, or `0` if none.
///
/// `$0` (final cursor) is excluded from the max since it's always last.
pub fn max_explicit_tabstop_index(content: &str) -> usize {
    collect_explicit_tabstop_indices(content)
        .into_iter()
        .filter(|index| *index > 0)
        .max()
        .unwrap_or(0)
}

/// Collect all numeric tabstop indices from VSCode snippet syntax.
fn collect_explicit_tabstop_indices(content: &str) -> Vec<usize> {
    let bytes = content.as_bytes();
    let mut indices = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != b'$' {
            i += 1;
            continue;
        }

        if i + 1 >= bytes.len() {
            break;
        }

        match bytes[i + 1] {
            // Escaped dollar — skip
            b'$' => {
                i += 2;
            }
            // Inline tabstop: $1, $23
            b'0'..=b'9' => {
                let (index, next_i) = parse_inline_index(bytes, i + 1);
                indices.push(index);
                i = next_i;
            }
            // Braced form: ${1:...}, ${1|...|}, ${0}
            b'{' => {
                if let Some((index, next_i)) = parse_braced_index(bytes, i + 2) {
                    indices.push(index);
                    i = next_i;
                } else {
                    // Not a numeric brace — skip (this is a named variable like ${name})
                    i += 2;
                }
            }
            _ => {
                i += 1;
            }
        }
    }

    indices
}

fn parse_inline_index(bytes: &[u8], start: usize) -> (usize, usize) {
    let mut value = 0usize;
    let mut i = start;

    while i < bytes.len() && bytes[i].is_ascii_digit() {
        value = value
            .saturating_mul(10)
            .saturating_add((bytes[i] - b'0') as usize);
        i += 1;
    }

    (value, i)
}

fn parse_braced_index(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    let mut value = 0usize;
    let mut i = start;
    let mut saw_digit = false;

    while i < bytes.len() && bytes[i].is_ascii_digit() {
        saw_digit = true;
        value = value
            .saturating_mul(10)
            .saturating_add((bytes[i] - b'0') as usize);
        i += 1;
    }

    if !saw_digit || i >= bytes.len() {
        return None;
    }

    // Must be followed by }, :, or | to be a valid tabstop
    if matches!(bytes[i], b'}' | b':' | b'|') {
        Some((value, i + 1))
    } else {
        None
    }
}
