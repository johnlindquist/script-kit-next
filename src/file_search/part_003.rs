/// Parse a directory path into its directory component and optional filter
///
/// This handles paths like:
/// - `~/dev/` -> directory=`~/dev/`, filter=None (list all)
/// - `~/dev/fin` -> directory=`~/dev/`, filter=Some("fin") (filter by "fin")
/// - `~/dev/mcp-` -> directory=`~/dev/`, filter=Some("mcp-") (filter by "mcp-")
/// - `/usr/local/bin` -> directory=`/usr/local/`, filter=Some("bin")
/// - `~` -> directory=`~`, filter=None
///
/// Returns None if:
/// - The path doesn't look like a directory path
/// - The parent directory doesn't exist
#[instrument(skip_all, fields(path = %path))]
pub fn parse_directory_path(path: &str) -> Option<ParsedDirPath> {
    let trimmed = path.trim();

    // Must be a directory-style path
    if !is_directory_path(trimmed) {
        return None;
    }

    // Handle special case: just ~ or ~/ (home directory)
    if trimmed == "~" || trimmed == "~/" {
        return Some(ParsedDirPath {
            directory: "~".to_string(),
            filter: None,
        });
    }

    // Handle paths ending with / - they're complete directory paths
    if trimmed.ends_with('/') {
        // Verify the directory exists
        if let Some(expanded) = expand_path(trimmed.trim_end_matches('/')) {
            let p = Path::new(&expanded);
            if p.is_dir() {
                return Some(ParsedDirPath {
                    directory: trimmed.to_string(),
                    filter: None,
                });
            }
        }
        return None;
    }

    // Path doesn't end with / - split into parent dir and potential filter
    // e.g., ~/dev/fin -> ~/dev/ + fin
    if let Some(last_slash_idx) = trimmed.rfind('/') {
        let parent = &trimmed[..=last_slash_idx]; // Include the slash
        let potential_filter = &trimmed[last_slash_idx + 1..];

        // Verify parent directory exists
        let parent_to_check = if parent == "/" {
            "/"
        } else {
            parent.trim_end_matches('/')
        };

        if let Some(expanded) = expand_path(parent_to_check) {
            let p = Path::new(&expanded);
            if p.is_dir() {
                let filter = if potential_filter.is_empty() {
                    None
                } else {
                    Some(potential_filter.to_string())
                };
                return Some(ParsedDirPath {
                    directory: parent.to_string(),
                    filter,
                });
            }
        }
    }

    None
}
/// List directory contents with optional filter applied
///
/// This combines directory listing with instant filtering for responsive UX.
/// When the user types `~/dev/fin`, we list `~/dev/` and filter by "fin".
///
/// # Arguments
/// * `dir_path` - Directory path (can include ~, ., ..)
/// * `filter` - Optional filter string to match against filenames
/// * `limit` - Maximum number of results to return
///
/// # Returns
/// Vector of FileResult structs matching the filter
#[allow(dead_code)]
#[instrument(skip_all, fields(dir_path = %dir_path, filter = ?filter, limit = limit))]
pub fn list_directory_filtered(
    dir_path: &str,
    filter: Option<&str>,
    limit: usize,
) -> Vec<FileResult> {
    // First get all directory contents
    let mut results = list_directory(dir_path, limit * 2); // Get more to filter from

    // Apply filter if present
    if let Some(filter_str) = filter {
        let filter_lower = filter_str.to_lowercase();
        results.retain(|r| r.name.to_lowercase().contains(&filter_lower));
    }

    // Apply limit after filtering
    results.truncate(limit);
    results
}
/// Filter and sort FileResults using Nucleo fuzzy matching
///
/// This function filters cached file results by fuzzy-matching the filter pattern
/// against file names, then sorts by match score (higher = better match).
///
/// # Arguments
/// * `results` - Slice of FileResult to filter
/// * `filter_pattern` - The pattern to fuzzy-match against file names
///
/// # Returns
/// Vector of (original_index, FileResult, score) tuples, sorted by score descending
#[allow(dead_code)]
pub fn filter_results_with_nucleo(
    results: &[FileResult],
    filter_pattern: &str,
) -> Vec<(usize, FileResult, u32)> {
    rank_file_results_nucleo(results, filter_pattern)
        .into_iter()
        .map(|(idx, score)| (idx, results[idx].clone(), score))
        .collect()
}
/// Core nucleo ranking helper returning only (index, score).
///
/// This keeps sorting/ranking allocations minimal and lets callers choose
/// whether they need owned copies or borrowed references.
fn rank_file_results_nucleo(results: &[FileResult], filter_pattern: &str) -> Vec<(usize, u32)> {
    use crate::scripts::NucleoCtx;

    let mut nucleo = NucleoCtx::new(filter_pattern);
    let mut scored: Vec<(usize, u32)> = results
        .iter()
        .enumerate()
        .filter_map(|(idx, r)| nucleo.score(&r.name).map(|score| (idx, score)))
        .collect();

    // Sort by score descending (higher = better match), then by name to
    // keep ranking deterministic when scores tie.
    scored.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| results[a.0].name.cmp(&results[b.0].name))
            .then_with(|| a.0.cmp(&b.0))
    });

    scored
}
/// Filter FileResults using Nucleo and return only (index, FileResult) pairs
///
/// This is a convenience wrapper for use in UI code where the score isn't needed.
/// Results are pre-sorted by match quality.
///
/// # Arguments
/// * `results` - Slice of FileResult to filter
/// * `filter_pattern` - The pattern to fuzzy-match against file names
///
/// # Returns
/// Vector of (original_index, &FileResult) tuples, sorted by match quality
#[allow(dead_code)]
pub fn filter_results_nucleo_simple<'a>(
    results: &'a [FileResult],
    filter_pattern: &str,
) -> Vec<(usize, &'a FileResult)> {
    rank_file_results_nucleo(results, filter_pattern)
        .into_iter()
        .map(|(idx, _)| (idx, &results[idx]))
        .collect()
}
