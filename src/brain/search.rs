//! Hybrid brain search: FTS5 BM25 + vector cosine fused with Reciprocal Rank
//! Fusion, then boosted by recent attention signals.
//!
//! This is the qmd retrieval recipe implemented natively: lexical search
//! catches exact terms, semantic search catches meaning, RRF makes them
//! agree, and signals tilt results toward what John currently cares about.

use super::store::{self, BrainDoc};
use anyhow::Result;
use std::collections::HashMap;

const RRF_K: f64 = 60.0;
const SIGNAL_WINDOW: usize = 200;
const SIGNAL_BOOST: f64 = 0.05;

#[derive(Debug, Clone)]
pub struct BrainHit {
    pub doc: BrainDoc,
    pub score: f64,
}

/// Rank documents for `query`. `query_vec` is the embedded query (None =>
/// lexical-only). Pure function over the supplied candidate lists — separated
/// from IO for testability.
pub(crate) fn fuse_ranks(
    fts_ids: &[i64],
    vec_ids: &[i64],
    signal_topics: &[(String, i64)],
    docs: &[BrainDoc],
    limit: usize,
) -> Vec<(i64, f64)> {
    let mut scores: HashMap<i64, f64> = HashMap::new();
    for (rank, id) in fts_ids.iter().enumerate() {
        *scores.entry(*id).or_default() += 1.0 / (RRF_K + rank as f64 + 1.0);
    }
    for (rank, id) in vec_ids.iter().enumerate() {
        *scores.entry(*id).or_default() += 1.0 / (RRF_K + rank as f64 + 1.0);
    }
    // Attention boost: docs whose title/content mention a recent signal topic
    // get a nudge proportional to the topic's recent weight.
    if !signal_topics.is_empty() {
        let by_id: HashMap<i64, &BrainDoc> = docs.iter().map(|d| (d.id, d)).collect();
        for (id, score) in scores.iter_mut() {
            if let Some(doc) = by_id.get(id) {
                let haystack = format!(
                    "{} {}",
                    doc.title.to_lowercase(),
                    doc.content.to_lowercase()
                );
                for (topic, weight) in signal_topics {
                    if haystack.contains(topic.as_str()) {
                        *score += SIGNAL_BOOST * (*weight as f64).min(3.0) / (RRF_K / 10.0);
                    }
                }
            }
        }
    }
    let mut ranked: Vec<(i64, f64)> = scores.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(limit);
    ranked
}

pub(crate) fn cosine_top_ids(
    query_vec: &[f32],
    embeddings: &[(i64, Vec<f32>)],
    limit: usize,
) -> Vec<i64> {
    let mut scored: Vec<(i64, f32)> = embeddings
        .iter()
        .filter(|(_, v)| v.len() == query_vec.len() && !v.is_empty())
        .map(|(id, v)| {
            let dot: f32 = query_vec.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
            (*id, dot)
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    scored.into_iter().map(|(id, _)| id).collect()
}

/// Aggregate recent signals into (topic, total weight), strongest first.
pub(crate) fn aggregate_signals(signals: &[store::BrainSignal]) -> Vec<(String, i64)> {
    let mut by_topic: HashMap<String, i64> = HashMap::new();
    for signal in signals {
        *by_topic.entry(signal.topic.clone()).or_default() += signal.weight;
    }
    let mut topics: Vec<(String, i64)> = by_topic
        .into_iter()
        .filter(|(topic, _)| topic.len() > 2)
        .collect();
    topics.sort_by(|a, b| b.1.cmp(&a.1));
    topics.truncate(16);
    topics
}

/// The main entry point: hybrid search over everything the brain knows.
/// `query_vec` should come from [`super::embedder::BrainEmbedder::embed`];
/// pass `None` for lexical-only (no model on disk, or latency-critical paths).
pub fn brain_search(
    query: &str,
    query_vec: Option<&[f32]>,
    model_id: Option<&str>,
    limit: usize,
) -> Result<Vec<BrainHit>> {
    let candidate_limit = limit.max(8) * 4;
    let fts_ids = store::fts_search(query, candidate_limit)?;
    let vec_ids = match (query_vec, model_id) {
        (Some(qv), Some(mid)) if !qv.is_empty() => {
            let embeddings = store::load_embeddings(mid)?;
            cosine_top_ids(qv, &embeddings, candidate_limit)
        }
        _ => Vec::new(),
    };
    let mut candidate_ids: Vec<i64> = fts_ids.clone();
    for id in &vec_ids {
        if !candidate_ids.contains(id) {
            candidate_ids.push(*id);
        }
    }
    let docs = store::get_docs_by_ids(&candidate_ids)?;
    let signals = store::recent_signals(SIGNAL_WINDOW).unwrap_or_default();
    let signal_topics = aggregate_signals(&signals);
    // Rank the full candidate pool, then dedupe by content before taking the
    // top `limit`: the same text captured via clipboard, a note, and a chat
    // turn must not crowd distinct memories out of the launcher section.
    let ranked = fuse_ranks(&fts_ids, &vec_ids, &signal_topics, &docs, candidate_limit);
    let by_id: HashMap<i64, BrainDoc> = docs.into_iter().map(|d| (d.id, d)).collect();
    let mut seen_content = std::collections::HashSet::new();
    Ok(ranked
        .into_iter()
        .filter_map(|(id, score)| by_id.get(&id).cloned().map(|doc| BrainHit { doc, score }))
        .filter(|hit| seen_content.insert(store::content_hash(&hit.doc.title, &hit.doc.content)))
        .take(limit)
        .collect())
}

/// Render hits as a compact markdown context block for agent prompts.
/// Hard-capped so retrieval can never blow out a prompt.
pub fn render_context_block(hits: &[BrainHit], max_chars: usize) -> String {
    if hits.is_empty() {
        return String::new();
    }
    let mut out = String::from(
        "## Brain recall (auto-retrieved from the user's local knowledge; \
         treat as background memory, cite naturally when relevant)\n\n",
    );
    for hit in hits {
        let title = if hit.doc.title.trim().is_empty() {
            "(untitled)"
        } else {
            hit.doc.title.trim()
        };
        let mut excerpt: String = hit.doc.content.chars().take(700).collect();
        if hit.doc.content.chars().count() > 700 {
            excerpt.push_str(" …");
        }
        let entry = format!(
            "### [{}] {}\n{}\n\n",
            hit.doc.source.label(),
            title,
            excerpt.trim()
        );
        if out.len() + entry.len() > max_chars {
            break;
        }
        out.push_str(&entry);
    }
    out
}
