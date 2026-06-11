//! Brain behavior tests. All run against a temp sqlite via
//! `SCRIPT_KIT_TEST_BRAIN_DB_PATH` (set per-process; tests share one DB and
//! use distinct source_ids).

use super::curator;
use super::inbox::{self, InboxKind};
use super::indexer::extract_topics;
use super::search::{aggregate_signals, cosine_top_ids, fuse_ranks};
use super::store::{self, BrainDoc, BrainSignal, DocSource};
use super::telegram;

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
fn topic_extraction_drops_conversational_filler() {
    // The exact failure mode that filled the inbox with "else"/"again":
    // a throwaway ask made of filler words must yield zero topics.
    let topics = extract_topics("can you do that again for something else a second time");
    assert!(
        topics.is_empty(),
        "filler-only ask must not record topics, got {topics:?}"
    );
    // Filler next to a real subject keeps the subject, drops the filler.
    let topics = extract_topics("try the thumbnail pipeline again");
    assert!(topics.iter().any(|t| t == "thumbnail pipeline"));
    assert!(!topics.iter().any(|t| t.contains("again")));
}

#[test]
fn substantive_topic_gate_accepts_subjects_and_rejects_filler() {
    use super::indexer::is_substantive_topic;
    for junk in ["again", "else", "second", "Something Else", "  ", "the"] {
        assert!(!is_substantive_topic(junk), "{junk:?} should be rejected");
    }
    for real in [
        "second brain",
        "build script",
        "YouTube pipeline",
        "egghead",
    ] {
        assert!(is_substantive_topic(real), "{real:?} should be accepted");
    }
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
fn prune_ages_out_old_journals_but_keeps_fresh_data() {
    init_test_db();
    let now = chrono::Utc::now().timestamp();
    // An ancient daily journal (well past the 90-day window)...
    store::upsert_doc(
        DocSource::Activity,
        "activity:2020-01-01",
        "Activity journal 2020-01-01",
        "prunable ancient action",
        now - 200 * 86_400,
    )
    .unwrap();
    // ...and the focus review, which shares the source but must survive
    // regardless of age (it's keyed without the `activity:` day prefix).
    store::upsert_doc(
        DocSource::Activity,
        "focus-review",
        "Focus review",
        "current focus distillation",
        now - 200 * 86_400,
    )
    .unwrap();
    store::record_signal("prune-test-topic", 1, "test").unwrap();

    let (journals, _signals, _inbox) = store::prune_ambient_data_at(now).unwrap();
    assert!(journals >= 1, "ancient journal should be pruned");
    assert!(
        store::get_doc(DocSource::Activity, "activity:2020-01-01")
            .unwrap()
            .is_none(),
        "ancient journal must be gone"
    );
    assert!(
        store::get_doc(DocSource::Activity, "focus-review")
            .unwrap()
            .is_some(),
        "focus review is exempt from journal pruning"
    );
    // Fresh signals survive a prune at the current time (guards against an
    // inverted cutoff wiping everything).
    let signals = store::recent_signals(500).unwrap();
    assert!(
        signals.iter().any(|s| s.topic == "prune-test-topic"),
        "fresh signals must survive pruning"
    );
}

#[test]
fn brain_status_resource_reports_health() {
    init_test_db();
    let (mime, body) = super::resources::read_brain_resource("kit://brain").unwrap();
    assert_eq!(mime, "application/json");
    let value: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(value.get("docsBySource").is_some(), "per-source counts");
    assert!(value.get("embedHelperFound").is_some(), "helper presence");
    assert!(value.get("lastIndexCycle").is_some(), "indexer heartbeat");
    assert_eq!(value["ftsVersion"], "2", "fts migration recorded");
    assert!(value["dbSizeBytes"].as_u64().unwrap_or(0) > 0, "db on disk");
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

#[test]
fn inbox_insert_dedupes_resolves_and_orders() {
    init_test_db();
    let first = inbox::insert_inbox_item(
        InboxKind::Commitment,
        "Inbox-test ship the gizmo",
        "said in chat yesterday",
        "chat_turn",
        "thread-inbox#1",
    )
    .unwrap();
    assert!(first, "first insert lands");
    // Same kind + title modulo case/whitespace → deduped.
    let dup = inbox::insert_inbox_item(
        InboxKind::Commitment,
        "  inbox-test SHIP   the gizmo ",
        "different detail",
        "chat_turn",
        "thread-inbox#2",
    )
    .unwrap();
    assert!(!dup, "normalized kind|title must dedupe");
    // Same title, different kind → distinct item.
    let other_kind = inbox::insert_inbox_item(
        InboxKind::Question,
        "Inbox-test ship the gizmo",
        "is this even a question",
        "chat_turn",
        "",
    )
    .unwrap();
    assert!(other_kind, "kind participates in the dedupe key");
    // Blank titles are rejected, not stored.
    assert!(!inbox::insert_inbox_item(InboxKind::Drift, "   ", "", "", "").unwrap());

    let open = inbox::open_inbox_items(1000).unwrap();
    let pos_question = open
        .iter()
        .position(|i| i.kind == InboxKind::Question && i.title == "Inbox-test ship the gizmo")
        .expect("question item open");
    let pos_commitment = open
        .iter()
        .position(|i| i.kind == InboxKind::Commitment && i.title == "Inbox-test ship the gizmo")
        .expect("commitment item open");
    assert!(pos_question < pos_commitment, "newest first");
    let commitment = &open[pos_commitment];
    assert_eq!(commitment.source, "chat_turn");
    assert_eq!(commitment.source_id, "thread-inbox#1", "first insert wins");
    assert!(commitment.resolved_at.is_none());
    assert!(inbox::count_open_inbox().unwrap() >= 2);

    assert!(inbox::resolve_inbox_item(commitment.id).unwrap());
    assert!(
        !inbox::resolve_inbox_item(commitment.id).unwrap(),
        "second resolve is a no-op"
    );
    let open = inbox::open_inbox_items(1000).unwrap();
    assert!(
        !open.iter().any(|i| i.id == commitment.id),
        "resolved item leaves the open list"
    );
}

#[test]
fn inbox_kind_roundtrips_and_labels() {
    for kind in [
        InboxKind::Commitment,
        InboxKind::Question,
        InboxKind::Drift,
        InboxKind::StalePin,
    ] {
        assert_eq!(InboxKind::parse(kind.as_str()), Some(kind));
        assert!(!kind.label().is_empty());
    }
    assert_eq!(InboxKind::parse("nonsense"), None);
    assert_eq!(InboxKind::StalePin.label(), "Stale Pin");
    assert_eq!(InboxKind::Question.label(), "Open Question");
    assert_eq!(InboxKind::Drift.label(), "Drifting");
}

#[test]
fn prune_removes_only_old_resolved_inbox_items() {
    init_test_db();
    let now = chrono::Utc::now().timestamp();
    // Tests share one DB and other tests may also run prune concurrently, so
    // assert end states (rows gone/kept) rather than this call's counts.
    inbox::insert_inbox_item(InboxKind::Drift, "Inbox-prune ancient resolved", "", "", "").unwrap();
    inbox::insert_inbox_item(InboxKind::Drift, "Inbox-prune fresh resolved", "", "", "").unwrap();
    inbox::insert_inbox_item(InboxKind::Drift, "Inbox-prune still open", "", "", "").unwrap();
    let open = inbox::open_inbox_items(1000).unwrap();
    let id_of = |title: &str| {
        open.iter()
            .find(|i| i.title == title)
            .unwrap_or_else(|| panic!("missing {title}"))
            .id
    };
    // Backdate one resolution past the 30-day retention window.
    assert!(
        inbox::resolve_inbox_item_at(id_of("Inbox-prune ancient resolved"), now - 40 * 86_400)
            .unwrap()
    );
    assert!(inbox::resolve_inbox_item_at(id_of("Inbox-prune fresh resolved"), now).unwrap());

    let (_journals, _signals, _inbox_removed) = store::prune_ambient_data_at(now).unwrap();

    // The aged resolved row is gone: its dedupe hash is freed, so the same
    // title inserts again.
    assert!(
        inbox::insert_inbox_item(InboxKind::Drift, "Inbox-prune ancient resolved", "", "", "")
            .unwrap(),
        "aged resolved item must be deleted"
    );
    // The freshly resolved row survives: its hash still blocks re-insertion.
    assert!(
        !inbox::insert_inbox_item(InboxKind::Drift, "Inbox-prune fresh resolved", "", "", "")
            .unwrap(),
        "recently resolved item must be retained"
    );
    // Open items are never pruned regardless of age.
    assert!(inbox::open_inbox_items(1000)
        .unwrap()
        .iter()
        .any(|i| i.title == "Inbox-prune still open"));
}

#[test]
fn inbox_response_prompt_includes_detail_and_source_context() {
    init_test_db();
    let source_id = "thread-inbox-response-prompt#2";
    store::upsert_doc(
        DocSource::ChatTurn,
        source_id,
        "Original chat turn",
        "User said the second option should be shipped after checking the migration notes.",
        200,
    )
    .unwrap();
    assert!(
        inbox::insert_inbox_item(
            InboxKind::Question,
            "Clarify the second option",
            "The conversation left open whether second means the script or the build script.",
            DocSource::ChatTurn.as_str(),
            source_id,
        )
        .unwrap(),
        "inbox item should insert"
    );

    let item = inbox::open_inbox_items(1000)
        .unwrap()
        .into_iter()
        .find(|item| item.source_id == source_id)
        .expect("inserted inbox item");
    let prompt = inbox::response_prompt_for_inbox_item(&item);

    assert!(prompt.contains("Follow up on this Brain Inbox item."));
    assert!(prompt.contains("- Type: Open Question"));
    assert!(prompt.contains("- Title: Clarify the second option"));
    assert!(prompt.contains(
        "- Details: The conversation left open whether second means the script or the build script."
    ));
    assert!(prompt.contains("- Source: chat_turn"));
    assert!(prompt.contains("- Source ID: thread-inbox-response-prompt#2"));
    assert!(prompt.contains("- Source title: Original chat turn"));
    assert!(prompt.contains("User said the second option should be shipped"));
    assert!(prompt.contains("Use the inbox details and source context above"));
}

#[test]
fn parse_inbox_extraction_accepts_strict_json() {
    let raw = r#"{"commitments":[{"title":"Ship the inbox","detail":"Promised in chat.","sourceId":"thread-9#2"}],"questions":[{"title":"Which DB?","detail":"Never answered.","sourceId":""}],"drift":[{"title":"YouTube pipeline","detail":"No activity in a week."}]}"#;
    let parsed = curator::parse_inbox_extraction(raw).unwrap();
    assert_eq!(parsed.commitments.len(), 1);
    assert_eq!(parsed.commitments[0].title, "Ship the inbox");
    assert_eq!(parsed.commitments[0].source_id, "thread-9#2");
    assert_eq!(parsed.questions.len(), 1);
    assert_eq!(parsed.questions[0].source_id, "");
    assert_eq!(parsed.drift.len(), 1);
    assert_eq!(parsed.drift[0].source_id, "", "drift omits sourceId");
}

#[test]
fn parse_inbox_extraction_tolerates_fences_and_prose() {
    let raw = "Sure! Here is the extraction you asked for:\n```json\n{\"commitments\":[],\"questions\":[{\"title\":\"What port?\",\"detail\":\"\",\"sourceId\":\"t#0\"}],\"drift\":[]}\n```\nLet me know if you need anything else.";
    let parsed = curator::parse_inbox_extraction(raw).unwrap();
    assert_eq!(parsed.questions.len(), 1);
    assert_eq!(parsed.questions[0].title, "What port?");
    assert!(parsed.commitments.is_empty());
}

#[test]
fn parse_inbox_extraction_rejects_garbage_and_caps_items() {
    assert!(
        curator::parse_inbox_extraction("no json anywhere").is_err(),
        "prose without an object is an error"
    );
    assert!(
        curator::parse_inbox_extraction("{this is not json}").is_err(),
        "malformed object is an error"
    );
    assert!(curator::parse_inbox_extraction("} backwards {").is_err());
    // Blank titles dropped; categories capped at 8.
    let many = (0..12)
        .map(|i| format!("{{\"title\":\"q{i}\",\"detail\":\"\"}}"))
        .collect::<Vec<_>>()
        .join(",");
    let raw = format!(
        "{{\"commitments\":[{{\"title\":\"   \",\"detail\":\"blank title\"}}],\"questions\":[{many}],\"drift\":[]}}"
    );
    let parsed = curator::parse_inbox_extraction(&raw).unwrap();
    assert!(parsed.commitments.is_empty(), "blank titles skipped");
    assert_eq!(parsed.questions.len(), 8, "per-category cap enforced");
}

#[test]
fn parse_inbox_extraction_gates_generic_drift_titles() {
    let raw = r#"{"commitments":[],"questions":[{"title":"What's going on today?","detail":"","sourceId":"t#0"}],"drift":[{"title":"else","detail":"attention 8, no activity"},{"title":"again","detail":""},{"title":"second brain","detail":"real subject survives"}]}"#;
    let parsed = curator::parse_inbox_extraction(raw).unwrap();
    let drift_titles: Vec<&str> = parsed.drift.iter().map(|i| i.title.as_str()).collect();
    assert_eq!(
        drift_titles,
        vec!["second brain"],
        "generic drift titles are filtered, substantive ones kept"
    );
    // The gate applies to drift only — a question phrased entirely in
    // generic words is still a genuine user question.
    assert_eq!(parsed.questions.len(), 1);
}

#[test]
fn stale_pin_detection_flags_old_pinned_notes_only() {
    let now = chrono::Utc::now();
    let old = now - chrono::Duration::days(30);
    let fresh = now - chrono::Duration::days(2);
    let notes = vec![
        ("Old pinned".to_string(), old, true, "note-old".to_string()),
        (
            "Fresh pinned".to_string(),
            fresh,
            true,
            "note-fresh".to_string(),
        ),
        (
            "Old unpinned".to_string(),
            old,
            false,
            "note-unpinned".to_string(),
        ),
        ("   ".to_string(), old, true, "note-untitled".to_string()),
    ];
    let stale = curator::stale_pins_from(&notes, now);
    assert_eq!(stale.len(), 2, "only old pinned notes qualify");
    let old_pin = stale
        .iter()
        .find(|(_, _, id)| id == "note-old")
        .expect("old pinned note flagged");
    assert_eq!(old_pin.0, "Old pinned");
    assert_eq!(
        old_pin.1,
        format!("Pinned but untouched since {}", old.format("%Y-%m-%d"))
    );
    let untitled = stale
        .iter()
        .find(|(_, _, id)| id == "note-untitled")
        .expect("untitled pinned note flagged");
    assert_eq!(untitled.0, "Pinned note", "blank title gets fallback");
    assert!(
        !stale
            .iter()
            .any(|(_, _, id)| id == "note-fresh" || id == "note-unpinned"),
        "fresh or unpinned notes are never flagged"
    );
}

// ============================================================
// Telegram bridge (pure core; no network)
// ============================================================

#[test]
fn telegram_parse_updates_extracts_messages_and_next_offset() {
    // 2 text messages + 1 non-message update (an edit): the messages route,
    // and the offset still advances past the non-message update.
    let body = r#"{
        "ok": true,
        "result": [
            {
                "update_id": 101,
                "message": {
                    "message_id": 7,
                    "from": {"id": 42, "is_bot": false, "first_name": "John"},
                    "chat": {"id": 42, "type": "private"},
                    "date": 1700000000,
                    "text": "what did I work on this week?"
                }
            },
            {
                "update_id": 102,
                "edited_message": {"message_id": 7, "text": "ignored edit"}
            },
            {
                "update_id": 103,
                "message": {
                    "message_id": 8,
                    "from": {"id": 7},
                    "chat": {"id": 7, "type": "private"},
                    "date": 1700000001,
                    "text": "/start"
                }
            }
        ]
    }"#;
    let updates = telegram::parse_updates_json(body).expect("realistic getUpdates parses");
    assert_eq!(updates.len(), 3, "non-message updates still parse");
    let messages = telegram::incoming_messages(&updates);
    assert_eq!(messages.len(), 2, "only text messages become incoming");
    assert_eq!(messages[0].update_id, 101);
    assert_eq!(messages[0].chat_id, 42);
    assert_eq!(messages[0].user_id, 42);
    assert_eq!(messages[0].text, "what did I work on this week?");
    assert_eq!(messages[1].text, "/start");
    assert_eq!(
        telegram::next_offset(&updates),
        Some(103),
        "offset covers the non-message update too"
    );
    assert_eq!(telegram::next_offset(&[]), None, "empty batch keeps offset");
}

#[test]
fn telegram_parse_tolerates_partial_messages_and_rejects_bad_envelopes() {
    assert!(telegram::parse_updates_json("not json").is_err());
    assert!(telegram::parse_updates_json(r#"{"ok": false, "result": []}"#).is_err());
    // A message missing text/from is skipped, but its id still advances.
    let updates = telegram::parse_updates_json(
        r#"{"ok": true, "result": [{"update_id": 5, "message": {"chat": {"id": 1}}}]}"#,
    )
    .expect("partial message parses");
    assert!(telegram::incoming_messages(&updates).is_empty());
    assert_eq!(telegram::next_offset(&updates), Some(5));
}

#[test]
fn telegram_authorization_requires_allowlist_membership() {
    assert!(telegram::is_authorized(42, &[42, 7]));
    assert!(!telegram::is_authorized(9, &[42, 7]));
    assert!(
        !telegram::is_authorized(42, &[]),
        "empty allowlist authorizes nobody"
    );
}

#[test]
fn telegram_answer_prompt_grounds_question_in_context() {
    let prompt =
        telegram::build_answer_prompt("what is project bluefin?", "[memory] bluefin notes");
    assert!(prompt.contains("what is project bluefin?"));
    assert!(prompt.contains("[memory] bluefin notes"));
    assert!(
        prompt.contains("ONLY"),
        "prompt restricts to memory context"
    );
    assert!(
        prompt.contains("no markdown"),
        "telegram replies are plain text"
    );
}

#[test]
fn telegram_replies_trim_and_truncate_at_cap() {
    assert_eq!(telegram::truncate_reply("  short answer  "), "short answer");
    let long = "x".repeat(5_000);
    let capped = telegram::truncate_reply(&long);
    assert_eq!(capped.chars().count(), 4_000, "capped under telegram limit");
    assert!(capped.ends_with('…'), "truncation is marked");
}

#[test]
fn telegram_redaction_strips_token_from_error_text() {
    let redacted = telegram::redact_token(
        "123456:ABC-secret",
        "https://api.telegram.org/bot123456:ABC-secret/getUpdates: timeout",
    );
    assert!(!redacted.contains("123456:ABC-secret"));
    assert!(redacted.contains("<redacted-token>"));
    assert_eq!(telegram::redact_token("", "unchanged"), "unchanged");
}

#[test]
fn capture_stores_sync_into_brain_and_respect_deletion() {
    init_test_db();
    let tmp = tempfile::TempDir::new().expect("tempdir");

    // Todos: one open, one already deleted.
    let menu_syntax_dir = tmp.path().join("menu-syntax");
    std::fs::create_dir_all(&menu_syntax_dir).expect("mkdir");
    std::fs::write(
        menu_syntax_dir.join("todos.jsonl"),
        concat!(
            r#"{"schema":"menu-syntax.todo.v1","kind":"todo","id":"todo_open","body":"Renew passport","status":"open","tags":["errands"],"updatedAt":"2026-06-01T10:00:00Z","deletedAt":null}"#,
            "\n",
            r#"{"schema":"menu-syntax.todo.v1","kind":"todo","id":"todo_gone","body":"Old thing","status":"deleted","updatedAt":"2026-06-01T10:00:00Z","deletedAt":"2026-06-02T10:00:00Z"}"#,
            "\n",
        ),
    )
    .expect("seed todos");

    // Link + snippet through their real stores (same writers the `;` capture
    // path uses).
    let link = match crate::menu_syntax::capture::parse_capture(
        ";link https://example.com Example description:Docs",
    ) {
        crate::menu_syntax::capture::CaptureParse::Ok(invocation) => invocation,
        _ => panic!("link capture should parse"),
    };
    let draft = crate::menu_syntax::parse_link_scriptlet_capture(&link).expect("link draft");
    crate::scriptlets::link_markdown_store::upsert_link_section(tmp.path(), &draft)
        .expect("write link");
    let snippet =
        match crate::menu_syntax::capture::parse_capture(";snippet Hello there keyword:hi name:Hi")
        {
            crate::menu_syntax::capture::CaptureParse::Ok(invocation) => invocation,
            _ => panic!("snippet capture should parse"),
        };
    let draft =
        crate::menu_syntax::parse_snippet_scriptlet_capture(&snippet).expect("snippet draft");
    crate::scriptlets::snippet_markdown_store::upsert_snippet_section(tmp.path(), &draft)
        .expect("write snippet");

    let synced = super::indexer::sync_capture_stores_in_sk_path(tmp.path()).expect("sync");
    assert_eq!(
        synced, 3,
        "open todo + link + snippet (deleted todo skipped)"
    );

    let todo = store::get_doc(DocSource::Capture, "todo:todo_open")
        .unwrap()
        .expect("open todo mirrored");
    assert!(todo.content.contains("Renew passport"));
    assert!(todo.content.contains("errands"));
    assert!(
        store::get_doc(DocSource::Capture, "todo:todo_gone")
            .unwrap()
            .is_none(),
        "deleted todo must not enter the brain"
    );
    let docs = store::recent_docs_for_source(DocSource::Capture, 0, 10).unwrap();
    assert!(docs.iter().any(|d| d.title == "Link: Example"));
    assert!(docs.iter().any(|d| d.title.starts_with("Snippet:")));

    // Deleting the todo from its store must make the brain forget it.
    std::fs::write(menu_syntax_dir.join("todos.jsonl"), "").expect("clear todos");
    super::indexer::sync_capture_stores_in_sk_path(tmp.path()).expect("resync");
    assert!(
        store::get_doc(DocSource::Capture, "todo:todo_open")
            .unwrap()
            .is_none(),
        "brain forgets a todo the user erased"
    );
}
