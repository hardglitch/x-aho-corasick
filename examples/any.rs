fn main() {
    let patterns = vec!["ma", "mama", "cat", "at", "ма", "кошка", "лужа", "каша"];
    let matcher = aho_corasick::FastPatternMatcher::new(&patterns);

    let text = "Мама (mama) вела кошку (cat) к луже, а в каше была (was) малина!".to_lowercase();
    let one_match = matcher.find_any_in(&text);
    let test_data = [(4,0)];

    // Start is a position in the byte array (text.as_bytes()). It's not a char position in the text
    for (n, (start, idx)) in one_match.into_iter().enumerate() {
        assert_eq!(test_data.get(n), Some(&(idx, start)));
        println!("Found '{}' at position {}", patterns.get(idx).unwrap_or(&"None"), start);
    }
}
