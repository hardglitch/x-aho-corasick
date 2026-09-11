use crate::{FastPatternMatcher, Match};

#[test]
fn test_find_all_non_overlapping1_pos() {
    let patterns = ["ma", "mama", "cat", "at", "ма", "кошка", "лужа", "каша"];
    let matcher = FastPatternMatcher::new(&patterns);

    let text = "Мама (mama) вела кошку (cat) к луже, а в каше была (was) малина (macatma)!".to_lowercase();
    let all_matches = matcher.find_all_in(&text); // non-overlapping

    let test_data = [
        Match::new(0, 4),
        Match::new(4, 4),
        Match::new(10, 0),
        Match::new(12, 0),
        Match::new(37, 2),
        Match::new(85, 4),
        Match::new(99, 0),
        Match::new(101, 2),
        Match::new(104, 0),
    ];

    for (n, m) in all_matches.enumerate() {
        assert_eq!(test_data.get(n), Some(&m));
    }
}

#[test]
fn test_find_all_non_overlapping2_pos() {
    let patterns = ["aa", "aaaa"];
    let matcher = FastPatternMatcher::new(&patterns);

    let text = "aaaa".to_lowercase();
    let all_matches = matcher.find_all_in(&text);

    let test_data = [
        Match::new(0, 0),
        Match::new(2, 0),
    ];

    for (n, m) in all_matches.enumerate() {
        assert_eq!(test_data.get(n), Some(&m));
    }
}

#[test]
fn test_find_any1_pos() {
    let patterns = ["ma", "mama", "cat", "at", "ма", "кошка", "лужа", "каша"];
    let matcher = FastPatternMatcher::new(&patterns);

    let text = "Мама (mama) вела кошку (cat) к луже, а в каше была (was) малина!".to_lowercase();
    let one_match = matcher.find_any_in(&text).take();
    let test_data = Some(Match::new(0, 4));

	assert_eq!(test_data, one_match);
}

#[test]
fn test_find_all_overlapping1_pos() {
    let patterns = ["mama", "ma", "cat", "at", "ма", "кошка", "лужа", "каша", "ca"];
    let matcher = FastPatternMatcher::new(&patterns);

    let text = "Мама (mama) вела кошку (cat) к луже, а в каше была (was) малина (macatma)!".to_lowercase();
    let all_matches = matcher.find_all_overlapping_in(&text);

    let test_data = [
        Match::new(0, 4),
        Match::new(4, 4),
        Match::new(10, 1),
        Match::new(10, 0),
        Match::new(12, 1),
        Match::new(37, 8),
        Match::new(37, 2),
        Match::new(38, 3),
        Match::new(85, 4),
        Match::new(99, 1),
        Match::new(101, 8),
        Match::new(101, 2),
        Match::new(102, 3),
        Match::new(104, 1),
    ];

    for (n, m) in all_matches.enumerate() {
        assert_eq!(test_data.get(n), Some(&m));
    }
}

#[test]
fn test_find_all_overlapping2_pos() {
    let patterns = ["aa".as_bytes(), "aaaa".as_bytes()];
    let matcher = FastPatternMatcher::new(&patterns);

    let text = "aaaa".to_lowercase();
    let all_matches = matcher.find_all_overlapping_in(&text);

    let test_data = [
        Match::new(0, 0),
        Match::new(1, 0),
        Match::new(0, 1),
        Match::new(2, 0),
    ];

    for (n, m) in all_matches.enumerate() {
        assert_eq!(test_data.get(n), Some(&m));
    }
}

#[test]
fn test_empty_text() {
    let patterns = ["abc".as_bytes(), "a".as_bytes()];
    let matcher = FastPatternMatcher::new(&patterns);
    let text = "";

    assert!(matcher.find_any_in(text).is_none());
    assert_eq!(matcher.find_all_in(text).count(), 0);
}

#[test]
fn test_no_matches() {
    let patterns = ["abc", "def"];
    let matcher = FastPatternMatcher::new(&patterns);
    let text = "xyz";

    assert!(matcher.find_any_in(text).is_none());
    assert_eq!(matcher.find_all_in(text).count(), 0);
}

#[test]
fn test_overlapping_nested_patterns() {
    let patterns = ["abcde", "bcd", "cd", "d"];
    let matcher = FastPatternMatcher::new(&patterns);
    let text = "abcde";

    let mut results: Vec<Match> = matcher.find_all_overlapping_in(text).collect();
    results.sort_by_key(|m| m.start());

    assert_eq!(results.len(), 4);
    assert_eq!(results[0].pattern(), 0); // abcde
    assert_eq!(results[1].pattern(), 1); //  bcd
    assert_eq!(results[2].pattern(), 2); //   cd
    assert_eq!(results[3].pattern(), 3); //    d
}

#[test]
fn test_duplicate_patterns1() {
    let patterns = ["abc", "abc"];
    let matcher = FastPatternMatcher::new(&patterns);
    let text = "abc";
    let all_matches = matcher.find_all_overlapping_in(text).collect::<Vec<_>>();
    assert_eq!(all_matches.len(), 1);
}

#[test]
fn test_duplicate_patterns2() {
    let patterns = ["abc", "ab", "abc", "bc"];
    let matcher = FastPatternMatcher::new(&patterns);
    let text = "abc";
    let all_matches = matcher.find_all_overlapping_in(text).collect::<Vec<_>>();
    let test_data = [
        Match::new(0, 1),
        Match::new(0, 2),
        Match::new(1, 3),
    ];
    assert_eq!(all_matches.len(), 3);
    assert_eq!(all_matches, test_data);
}

#[test]
fn test_non_overlapping_greedy() {
    let patterns = ["aa".as_bytes()];
    let matcher = FastPatternMatcher::new(&patterns);
    let text = "aaaaa";

    let matches = matcher.find_all_in(text).collect::<Vec<_>>();
    assert_eq!(matches.len(), 2); // [0, 2]
}

#[test]
fn test_patterns_with_special_chars() {
    // Проверка работы с пробелами и спецсимволами
    let patterns = ["a b".to_string(), "c d".to_string()];
    let matcher = FastPatternMatcher::new(&patterns);
    let text = "a b c d";

    let all_matches = matcher.find_all_overlapping_in(text).collect::<Vec<_>>();
    assert_eq!(all_matches.len(), 2);
}

#[test]
fn test_large_alphabet_boundaries() {
    // "🦀" in UTF-8 - [240, 159, 166, 128]
    let patterns = ["🦀".as_bytes()];
    let matcher = FastPatternMatcher::new(&patterns);
    let text = "crab 🦀 crab 🦀";

    let all_matches = matcher.find_all_overlapping_in(text).collect::<Vec<_>>();
    assert_eq!(all_matches.len(), 2);
}
