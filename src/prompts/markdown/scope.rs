use super::*;

pub(super) static MARKDOWN_CACHE: OnceLock<Mutex<HashMap<u64, Arc<Vec<ParsedBlock>>>>> =
    OnceLock::new();
pub(super) static MARKDOWN_VOLATILE_SCOPE_COUNTER: AtomicU64 = AtomicU64::new(1);
pub(super) const INFERRED_SCOPE_PREFIX_CHARS: usize = 256;

pub(super) fn markdown_cache_key(text: &str, is_dark: bool) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    is_dark.hash(&mut hasher);
    hasher.finish()
}

pub(super) fn stable_markdown_scope_hash(scope: Option<&str>) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    match scope {
        Some(scope) => {
            "scoped".hash(&mut hasher);
            scope.hash(&mut hasher);
        }
        None => {
            // Unscoped renders need unique IDs to avoid collisions when the same
            // markdown appears in multiple places at once. These IDs are stable
            // only within a single render pass.
            let nonce = MARKDOWN_VOLATILE_SCOPE_COUNTER.fetch_add(1, Ordering::Relaxed);
            "volatile".hash(&mut hasher);
            nonce.hash(&mut hasher);
        }
    }
    hasher.finish()
}

pub(super) fn scoped_markdown_element_id(
    scope_hash: u64,
    kind: &str,
    primary_index: usize,
    secondary_index: usize,
) -> SharedString {
    SharedString::from(format!(
        "md-{scope_hash:016x}-{kind}-{primary_index}-{secondary_index}"
    ))
}

pub(super) fn scoped_markdown_numeric_key(
    scope_hash: u64,
    kind: &str,
    primary_index: usize,
    secondary_index: usize,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    scope_hash.hash(&mut hasher);
    kind.hash(&mut hasher);
    primary_index.hash(&mut hasher);
    secondary_index.hash(&mut hasher);
    hasher.finish()
}

pub(super) fn inferred_markdown_scope_hash(text: &str) -> u64 {
    let prefix_end = text
        .char_indices()
        .nth(INFERRED_SCOPE_PREFIX_CHARS)
        .map(|(idx, _)| idx)
        .unwrap_or(text.len());
    stable_markdown_scope_hash(Some(&text[..prefix_end]))
}
