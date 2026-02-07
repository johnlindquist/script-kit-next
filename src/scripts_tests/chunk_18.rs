// ============================================================================
// NUCLEO INTEGRATION TESTS
// ============================================================================
// These tests verify the nucleo_score helper function for fuzzy matching

#[test]
fn test_nucleo_score_basic_match() {
    use nucleo_matcher::pattern::Pattern;
    use nucleo_matcher::{Matcher, Utf32Str};

    // Test basic fuzzy matching
    let pattern = Pattern::parse(
        "hello",
        nucleo_matcher::pattern::CaseMatching::Smart,
        nucleo_matcher::pattern::Normalization::Smart,
    );
    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);

    // Score a matching haystack
    let mut buf = Vec::new();
    let haystack = Utf32Str::new("hello world", &mut buf);
    let score = pattern.score(haystack, &mut matcher);

    assert!(
        score.is_some(),
        "nucleo should match 'hello' in 'hello world'"
    );
    assert!(
        score.unwrap() > 0,
        "score should be positive for exact match"
    );
}

#[test]
fn test_nucleo_score_fuzzy_match() {
    use nucleo_matcher::pattern::Pattern;
    use nucleo_matcher::{Matcher, Utf32Str};

    let pattern = Pattern::parse(
        "hlo",
        nucleo_matcher::pattern::CaseMatching::Smart,
        nucleo_matcher::pattern::Normalization::Smart,
    );
    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);

    // Score a fuzzy matching haystack (h-e-l-l-o contains h-l-o)
    let mut buf = Vec::new();
    let haystack = Utf32Str::new("hello", &mut buf);
    let score = pattern.score(haystack, &mut matcher);

    assert!(
        score.is_some(),
        "nucleo should fuzzy match 'hlo' in 'hello'"
    );
}

#[test]
fn test_nucleo_score_no_match() {
    use nucleo_matcher::pattern::Pattern;
    use nucleo_matcher::{Matcher, Utf32Str};

    let pattern = Pattern::parse(
        "xyz",
        nucleo_matcher::pattern::CaseMatching::Smart,
        nucleo_matcher::pattern::Normalization::Smart,
    );
    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);

    let mut buf = Vec::new();
    let haystack = Utf32Str::new("hello world", &mut buf);
    let score = pattern.score(haystack, &mut matcher);

    assert!(
        score.is_none(),
        "nucleo should not match 'xyz' in 'hello world'"
    );
}

#[test]
fn test_nucleo_score_ranking() {
    use nucleo_matcher::pattern::Pattern;
    use nucleo_matcher::{Matcher, Utf32Str};

    let pattern = Pattern::parse(
        "git",
        nucleo_matcher::pattern::CaseMatching::Smart,
        nucleo_matcher::pattern::Normalization::Smart,
    );
    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);

    // Exact match should score higher than partial match
    let mut buf1 = Vec::new();
    let haystack_exact = Utf32Str::new("git-commit", &mut buf1);
    let score_exact = pattern.score(haystack_exact, &mut matcher);

    let mut buf2 = Vec::new();
    let haystack_partial = Utf32Str::new("digit-recognizer", &mut buf2);
    let score_partial = pattern.score(haystack_partial, &mut matcher);

    assert!(score_exact.is_some(), "should match 'git' in 'git-commit'");
    assert!(
        score_partial.is_some(),
        "should match 'git' in 'digit-recognizer'"
    );

    // Exact prefix should score higher
    assert!(
        score_exact.unwrap() > score_partial.unwrap(),
        "exact prefix 'git-commit' should score higher than 'digit-recognizer'"
    );
}

#[test]
fn test_nucleo_score_case_insensitive() {
    use nucleo_matcher::pattern::Pattern;
    use nucleo_matcher::{Matcher, Utf32Str};

    // Smart mode: lowercase pattern matches case-insensitively
    let pattern = Pattern::parse(
        "hello",
        nucleo_matcher::pattern::CaseMatching::Smart,
        nucleo_matcher::pattern::Normalization::Smart,
    );
    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);

    let mut buf = Vec::new();
    let haystack = Utf32Str::new("HELLO WORLD", &mut buf);
    let score = pattern.score(haystack, &mut matcher);

    // Lowercase pattern with Smart case matching should match uppercase haystack
    assert!(
        score.is_some(),
        "nucleo with Smart case matching should match lowercase 'hello' in uppercase 'HELLO WORLD'"
    );
}
