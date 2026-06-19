//! Markdown-aware chunking for embeddings — the qmd recipe: split long
//! documents into ~900-token pieces with ~15% overlap, preferring breaks at
//! headings, then blank lines, then line ends, then sentence ends, so each
//! chunk is a coherent unit of meaning. Without this, whole-doc embedding
//! truncates long day pages and notes and everything past the cap is
//! invisible to semantic search.
//!
//! There is no tokenizer on this side of the embedder boundary, so sizes are
//! byte budgets at ~4 bytes/token: 900 tokens ≈ 3600 bytes, 15% ≈ 540 bytes.

/// ~900 tokens at ~4 bytes/token.
pub const CHUNK_TARGET_BYTES: usize = 3_600;
/// ~15% overlap between consecutive chunks.
pub const CHUNK_OVERLAP_BYTES: usize = 540;
/// Pathological-input guard: one doc can never queue unbounded embed work.
/// 64 chunks ≈ 230 KB of text — far beyond any real note or day page.
pub const MAX_CHUNKS_PER_DOC: usize = 64;

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    /// Byte offset of the chunk start within the source text.
    pub start: usize,
    pub text: String,
}

/// Split `text` into embed-ready chunks. Short inputs (≤ target) come back as
/// a single chunk — the common fast path allocates once and never scans.
pub fn chunk_markdown(text: &str) -> Vec<Chunk> {
    chunk_markdown_with(text, CHUNK_TARGET_BYTES, CHUNK_OVERLAP_BYTES)
}

pub fn chunk_markdown_with(text: &str, target: usize, overlap: usize) -> Vec<Chunk> {
    let trimmed = text.trim_end();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if trimmed.len() <= target {
        return vec![Chunk {
            start: 0,
            text: trimmed.to_string(),
        }];
    }

    let target = target.max(256);
    let overlap = overlap.min(target / 2);
    let mut chunks = Vec::new();
    let mut start = 0usize;
    while start < trimmed.len() && chunks.len() < MAX_CHUNKS_PER_DOC {
        start = floor_char_boundary(trimmed, start);
        let hard_end = floor_char_boundary(trimmed, (start + target).min(trimmed.len()));
        let end = if hard_end == trimmed.len() {
            hard_end
        } else {
            best_break(trimmed, start, hard_end)
        };
        if end <= start {
            let next = next_char_boundary_after(trimmed, start);
            if next <= start || next >= trimmed.len() {
                break;
            }
            start = next;
            continue;
        }
        let piece = trimmed[start..end].trim_end();
        if !piece.is_empty() {
            chunks.push(Chunk {
                start,
                text: piece.to_string(),
            });
        }
        if end >= trimmed.len() {
            break;
        }
        // Step back `overlap` bytes from the break so context spans the seam,
        // but always advance past the previous start to guarantee progress.
        let overlap_start = floor_char_boundary(trimmed, end.saturating_sub(overlap));
        let min_next = next_char_boundary_after(trimmed, start);
        start = overlap_start.max(min_next);
    }
    chunks
}

/// Pick the best break point in `(min_end, hard_end]`, searching backward
/// from the hard cap: heading line > blank line > newline > sentence end >
/// space > hard cap. Only breaks in the back third are considered so chunks
/// stay near the target size.
fn best_break(text: &str, start: usize, hard_end: usize) -> usize {
    let start = floor_char_boundary(text, start);
    let hard_end = floor_char_boundary(text, hard_end);
    let window_start = start + (hard_end - start) * 2 / 3;
    let window = &text[..hard_end];

    // A heading begins a new section — break BEFORE the `#` line.
    if let Some(pos) = rfind_in(window, window_start, "\n#") {
        return pos + 1; // keep the `\n` with the previous chunk
    }
    if let Some(pos) = rfind_in(window, window_start, "\n\n") {
        return pos + 2;
    }
    if let Some(pos) = rfind_in(window, window_start, "\n") {
        return pos + 1;
    }
    if let Some(pos) = rfind_in(window, window_start, ". ") {
        return pos + 2;
    }
    if let Some(pos) = rfind_in(window, window_start, " ") {
        return pos + 1;
    }
    floor_char_boundary(text, hard_end)
}

fn rfind_in(window: &str, min_pos: usize, needle: &str) -> Option<usize> {
    window.rfind(needle).filter(|pos| *pos >= min_pos)
}

fn floor_char_boundary(text: &str, mut pos: usize) -> usize {
    pos = pos.min(text.len());
    while pos > 0 && !text.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

fn ceil_char_boundary(text: &str, mut pos: usize) -> usize {
    pos = pos.min(text.len());
    while pos < text.len() && !text.is_char_boundary(pos) {
        pos += 1;
    }
    pos
}

fn next_char_boundary_after(text: &str, pos: usize) -> usize {
    ceil_char_boundary(text, pos.saturating_add(1)).min(text.len())
}

#[cfg(test)]
mod chunker_tests {
    use super::*;

    #[test]
    fn empty_input_yields_no_chunks() {
        assert!(chunk_markdown("").is_empty());
        assert!(chunk_markdown("   \n  ").is_empty());
    }

    #[test]
    fn short_doc_is_one_chunk() {
        let chunks = chunk_markdown("# Title\n\nA short note.");
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].start, 0);
        assert_eq!(chunks[0].text, "# Title\n\nA short note.");
    }

    #[test]
    fn long_doc_chunks_overlap_and_cover_everything() {
        let para = "Some sentence about the project that keeps going. ".repeat(8);
        let doc = (0..20)
            .map(|i| format!("## Section {i}\n\n{para}"))
            .collect::<Vec<_>>()
            .join("\n\n");
        let chunks = chunk_markdown_with(&doc, 1_000, 150);
        assert!(chunks.len() > 3, "long doc must split: {}", chunks.len());
        // Coverage: every chunk starts at or before the previous chunk's end
        // (overlap), and the last chunk reaches the end of the doc.
        for pair in chunks.windows(2) {
            let prev_end = pair[0].start + pair[0].text.len();
            assert!(
                pair[1].start <= prev_end,
                "gap between chunks: {} > {}",
                pair[1].start,
                prev_end
            );
            assert!(pair[1].start > pair[0].start, "chunks must advance");
        }
        let last = chunks.last().unwrap();
        assert_eq!(last.start + last.text.len(), doc.trim_end().len());
    }

    #[test]
    fn breaks_prefer_heading_boundaries() {
        // The break lands ON the heading line: the first chunk carries only
        // the section BEFORE the heading; the heading travels with its body
        // in the next chunk (whose start may rewind further for overlap).
        let filler = "word ".repeat(150); // ~750 bytes
        let doc = format!("{filler}\n## Heading\n{filler}");
        let chunks = chunk_markdown_with(&doc, 1_000, 100);
        assert!(chunks.len() >= 2);
        assert!(
            !chunks[0].text.contains("## Heading"),
            "first chunk must break before the heading"
        );
        assert!(
            chunks[1].text.contains("## Heading"),
            "heading stays with its section body"
        );
    }

    #[test]
    fn multibyte_text_never_panics() {
        let doc = "héllo wörld 🎉 ".repeat(400);
        let chunks = chunk_markdown_with(&doc, 1_000, 150);
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(!chunk.text.is_empty());
        }
    }

    #[test]
    fn hard_end_inside_em_dash_never_panics() {
        let doc = format!("{}—{}", "a".repeat(255), " tail ".repeat(500));
        let chunks = chunk_markdown_with(&doc, 256, 32);

        assert!(!chunks.is_empty());
        assert_eq!(
            chunks.last().unwrap().start + chunks.last().unwrap().text.len(),
            doc.trim_end().len()
        );
        for chunk in &chunks {
            assert!(doc.is_char_boundary(chunk.start));
            assert!(doc.is_char_boundary(chunk.start + chunk.text.len()));
        }
    }

    #[test]
    fn overlap_start_inside_multibyte_still_advances() {
        let doc = "alpha — beta ".repeat(500);
        let chunks = chunk_markdown_with(&doc, 257, 129);

        assert!(!chunks.is_empty());
        for pair in chunks.windows(2) {
            assert!(pair[1].start > pair[0].start, "chunks must advance");
            assert!(doc.is_char_boundary(pair[1].start));
        }
    }

    #[test]
    fn pathological_doc_is_capped() {
        let doc = "x".repeat(CHUNK_TARGET_BYTES * (MAX_CHUNKS_PER_DOC + 10));
        let chunks = chunk_markdown(&doc);
        assert_eq!(chunks.len(), MAX_CHUNKS_PER_DOC);
    }
}
