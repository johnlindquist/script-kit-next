//! Brain behavior tests. All run against a temp sqlite via
//! `SCRIPT_KIT_TEST_BRAIN_DB_PATH` (set per-process; tests share one DB and
//! use distinct source_ids).

use super::indexer::extract_topics;
use super::search::{aggregate_signals, cosine_top_ids, fuse_ranks};
use super::store::{self, BrainDoc, BrainSignal, DocSource};

fn init_test_db() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let path = std::env::temp_dir().join(format!(
            "brain-test-{}-{}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or_default()
        ));
        std::env::set_var("SCRIPT_KIT_TEST_BRAIN_DB_PATH", &path);
        store::init_brain_db().expect("init test brain db");
    });
}

fn doc(id: i64, title: &str, content: &str) -> BrainDoc {
    BrainDoc {
        id,
        source: DocSource::Note,
        source_id: id.to_string(),
        title: title.to_string(),
        content: content.to_string(),
        updated_at: 0,
    }
}

#[test]
fn upsert_is_idempotent_and_updates_on_change() {
    init_test_db();
    let id1 = store::upsert_doc(DocSource::Note, "n-upsert", "Title", "body", 100).unwrap();
    let id2 = store::upsert_doc(DocSource::Note, "n-upsert", "Title", "body", 100).unwrap();
    assert_eq!(id1, id2, "same source_id must keep one row");
    let id3 = store::upsert_doc(DocSource::Note, "n-upsert", "Title", "body v2", 200).unwrap();
    assert_eq!(id1, id3);
    let docs = store::get_docs_by_ids(&[id1]).unwrap();
    assert_eq!(docs[0].content, "body v2");
}

#[test]
fn fts_matches_natural_language_questions() {
    init_test_db();
    let id = store::upsert_doc(
        DocSource::Note,
        "n-nlq",
        "Project Bluefin launch checklist",
        "Project Bluefin deploys from the zephyr-42 branch.",
        100,
    )
    .unwrap();
    // Question contains filler words absent from the doc — OR semantics must
    // still surface it.
    let hits = store::fts_search("what branch does project bluefin deploy from", 10).unwrap();
    assert!(hits.contains(&id), "natural-language question should match");
}

#[test]
fn fts_finds_doc_by_content_and_respects_deletion() {
    init_test_db();
    let id = store::upsert_doc(
        DocSource::Note,
        "n-fts",
        "Postgres tricks",
        "use EXPLAIN ANALYZE to find slow queries",
        100,
    )
    .unwrap();
    let hits = store::fts_search("explain analyze slow", 10).unwrap();
    assert!(hits.contains(&id), "fts should match content terms");
    store::remove_doc(DocSource::Note, "n-fts").unwrap();
    let hits = store::fts_search("explain analyze slow", 10).unwrap();
    assert!(!hits.contains(&id), "fts must drop deleted docs");
}

#[test]
fn embedding_roundtrip_and_staleness() {
    init_test_db();
    let id = store::upsert_doc(DocSource::Note, "n-embed", "T", "v1", 100).unwrap();
    let pending = store::docs_needing_embedding("model-a", 50).unwrap();
    assert!(
        pending.iter().any(|d| d.id == id),
        "new doc needs embedding"
    );
    store::store_embedding(id, "model-a", "T", "v1", &[0.6, 0.8]).unwrap();
    let pending = store::docs_needing_embedding("model-a", 50).unwrap();
    assert!(
        !pending.iter().any(|d| d.id == id),
        "embedded doc is current"
    );
    // Content change invalidates by hash.
    store::upsert_doc(DocSource::Note, "n-embed", "T", "v2", 200).unwrap();
    let pending = store::docs_needing_embedding("model-a", 50).unwrap();
    assert!(
        pending.iter().any(|d| d.id == id),
        "changed doc needs re-embed"
    );
    // Different model invalidates everything.
    store::store_embedding(id, "model-a", "T", "v2", &[1.0, 0.0]).unwrap();
    let pending = store::docs_needing_embedding("model-b", 50).unwrap();
    assert!(
        pending.iter().any(|d| d.id == id),
        "model swap needs re-embed"
    );
    let loaded = store::load_embeddings("model-a").unwrap();
    let vec = &loaded.iter().find(|(i, _)| *i == id).unwrap().1;
    assert_eq!(vec, &vec![1.0, 0.0], "blob roundtrip preserves f32s");
}

#[test]
fn cosine_orders_by_similarity() {
    let embeddings = vec![
        (1, vec![1.0, 0.0]),
        (2, vec![0.0, 1.0]),
        (3, vec![0.9, 0.1]),
    ];
    let top = cosine_top_ids(&[1.0, 0.0], &embeddings, 2);
    assert_eq!(top, vec![1, 3]);
}

#[test]
fn rrf_prefers_docs_ranked_by_both_systems() {
    let docs = vec![doc(1, "a", ""), doc(2, "b", ""), doc(3, "c", "")];
    // doc 2 is mid-ranked by both; doc 1 only lexical; doc 3 only semantic.
    let ranked = fuse_ranks(&[1, 2], &[3, 2], &[], &docs, 10);
    assert_eq!(ranked[0].0, 2, "agreement between rankers wins");
}

#[test]
fn signals_boost_matching_docs() {
    let docs = vec![
        doc(1, "random note", "nothing relevant"),
        doc(2, "youtube strategy", "thumbnails and hooks"),
    ];
    let no_boost = fuse_ranks(&[1, 2], &[], &[], &docs, 10);
    assert_eq!(no_boost[0].0, 1);
    let boosted = fuse_ranks(&[1, 2], &[], &[("youtube".to_string(), 6)], &docs, 10);
    assert_eq!(boosted[0].0, 2, "attention signal should re-rank");
}

#[test]
fn signal_recording_and_aggregation() {
    init_test_db();
    store::record_signal("script kit", 2, "ask").unwrap();
    store::record_signal("Script Kit", 1, "chat").unwrap();
    store::record_signal("", 5, "ask").unwrap(); // ignored
    let signals = store::recent_signals(50).unwrap();
    let aggregated = aggregate_signals(&signals);
    let entry = aggregated.iter().find(|(t, _)| t == "script kit");
    assert!(
        entry.is_some_and(|(_, w)| *w >= 3),
        "weights accumulate case-insensitively"
    );
}

#[test]
fn aggregate_orders_by_weight() {
    let signals = vec![
        BrainSignal {
            topic: "alpha".into(),
            weight: 1,
            source: "ask".into(),
            created_at: 0,
        },
        BrainSignal {
            topic: "beta".into(),
            weight: 5,
            source: "ask".into(),
            created_at: 0,
        },
        BrainSignal {
            topic: "alpha".into(),
            weight: 1,
            source: "chat".into(),
            created_at: 0,
        },
    ];
    let aggregated = aggregate_signals(&signals);
    assert_eq!(aggregated[0].0, "beta");
}

#[test]
fn topic_extraction_skips_stopwords_and_keeps_pairs() {
    let topics = extract_topics("How do I configure the YouTube thumbnail pipeline?");
    assert!(topics.iter().any(|t| t.contains("youtube")));
    assert!(!topics.iter().any(|t| t == "how" || t == "the"));
}

#[test]
fn chat_turn_ingestion_is_idempotent_and_searchable() {
    init_test_db();
    super::ingest_chat_turn(
        "thread-x",
        0,
        "how do I deploy the egghead site",
        "Use vercel --prod.",
    )
    .unwrap();
    super::ingest_chat_turn(
        "thread-x",
        0,
        "how do I deploy the egghead site",
        "Use vercel --prod.",
    )
    .unwrap();
    let hits = store::fts_search("egghead deploy", 10).unwrap();
    let docs = store::get_docs_by_ids(&hits).unwrap();
    let matching: Vec<_> = docs
        .iter()
        .filter(|d| d.source == DocSource::ChatTurn && d.source_id == "thread-x#0")
        .collect();
    assert_eq!(matching.len(), 1, "same turn ingested once");
}

#[test]
fn retain_docs_forgets_deleted_sources() {
    init_test_db();
    // Uses the Clipboard source: tests share one DB and run in parallel, and
    // retention is destructive within its source — Note docs belong to other
    // tests.
    store::upsert_doc(
        DocSource::Clipboard,
        "clip-keep",
        "Keeper",
        "clipstays",
        100,
    )
    .unwrap();
    store::upsert_doc(
        DocSource::Clipboard,
        "clip-gone",
        "Goner",
        "clipleaves",
        100,
    )
    .unwrap();
    store::upsert_doc(
        DocSource::ChatTurn,
        "chat-keep-ret",
        "Chat",
        "clipstays too",
        100,
    )
    .unwrap();
    let removed = store::retain_docs(DocSource::Clipboard, &["clip-keep".to_string()]).unwrap();
    assert!(removed >= 1);
    let hits = store::fts_search("clipleaves", 10).unwrap();
    let docs = store::get_docs_by_ids(&hits).unwrap();
    assert!(
        !docs.iter().any(|d| d.source_id == "clip-gone"),
        "deleted source must be forgotten"
    );
    let chat_hits = store::fts_search("clipstays", 20).unwrap();
    let chat_docs = store::get_docs_by_ids(&chat_hits).unwrap();
    assert!(
        chat_docs.iter().any(|d| d.source_id == "chat-keep-ret"),
        "other sources untouched"
    );
}

#[test]
fn activity_journal_appends_newest_first_and_recalls() {
    init_test_db();
    store::append_activity("searched files for \"png\" and opened CleanShot.png").unwrap();
    store::append_activity("ran script kill-port").unwrap();
    let hits = store::fts_search("what was the last thing I searched for", 10).unwrap();
    let docs = store::get_docs_by_ids(&hits).unwrap();
    let journal = docs
        .iter()
        .find(|d| d.source == DocSource::Activity)
        .expect("activity journal should match a 'searched' question");
    // Newest first: the script run line must appear before the search line.
    let script_pos = journal.content.find("kill-port").unwrap();
    let search_pos = journal.content.find("CleanShot").unwrap();
    assert!(script_pos < search_pos, "journal must be newest-first");
    // One doc per day, not one per event.
    let all =
        store::get_docs_by_ids(&store::fts_search("kill-port CleanShot", 50).unwrap()).unwrap();
    let journals: Vec<_> = all
        .iter()
        .filter(|d| d.source == DocSource::Activity)
        .collect();
    assert_eq!(journals.len(), 1);
    // And recall renders it.
    let block = super::recall_context_block("what did I search for recently").unwrap();
    assert!(block.is_some_and(|b| b.contains("CleanShot")));
}

#[test]
fn recall_context_block_formats_and_caps() {
    init_test_db();
    store::upsert_doc(
        DocSource::Note,
        "n-recall",
        "Egghead publish checklist",
        "vercel --prod then update the course index",
        100,
    )
    .unwrap();
    let block = super::recall_context_block("egghead publish checklist").unwrap();
    let block = block.expect("expected recall content");
    assert!(block.contains("Brain recall"));
    assert!(block.contains("Egghead publish checklist"));
    assert!(block.len() <= super::BRAIN_CONTEXT_MAX_CHARS + 200);
    // Irrelevant queries return None, not noise.
    let none = super::recall_context_block("zzqx unrelated nonsense").unwrap();
    assert!(none.is_none());
}
