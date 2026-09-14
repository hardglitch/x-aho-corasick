use x_aho_corasick::Match;

fn main() {
    let patterns = ["ma", "mama", "cat", "at", "ма", "кошка", "лужа", "каша"];
    let matcher = x_aho_corasick::FastPatternMatcher::new(&patterns);

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

    // Start is a position in the byte array (text.as_bytes()). It's not a char position in the text
    for (n, m) in all_matches.enumerate() {
        assert_eq!(test_data.get(n), Some(&m));
        println!("Found '{}' at position {}", patterns[m.pattern_index()], m.start());
    }
}