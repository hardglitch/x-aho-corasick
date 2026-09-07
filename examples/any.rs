fn main() {
    let patterns = vec!["ma", "mama", "cat", "at", "ма", "кошка", "лужа", "каша"];
    let matcher = x_aho_corasick::FastPatternMatcher::new(&patterns);

    let text = "Мама (mama) вела кошку (cat) к луже, а в каше была (was) малина!".to_lowercase();
    let one_match = matcher.find_any_in(&text);
    let test_data = [(4,0)];

    // Start is a position in the byte array (text.as_bytes()). It's not a char position in the text
    for (n, m) in one_match.into_iter().enumerate() {
        assert_eq!(test_data.get(n), Some(&(m.pattern_index(), m.start_pos())));
        println!("Found '{}' at position {}", patterns.get(m.pattern_index()).unwrap_or(&"None"), m.start_pos());
    }
}
