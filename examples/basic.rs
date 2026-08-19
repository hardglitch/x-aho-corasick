use aho_corasick::FastPatternMatcher;

fn main() {
    let patterns = vec!["ma", "mama", "cat", "at", "ма", "кошка", "лужа", "каша"];
    let matcher = FastPatternMatcher::new(&patterns);

    let text = "Мама (mama) вела кошку (cat) к луже, а в каше была (was) малина!".to_lowercase();
    let matches = matcher.find_all(&text);

    let test_data = [(4,0), (4,4), (0,10), (1,10), (0,12), (2,37), (3,38), (4,85)];

    // Start is a position in the byte array (text.as_bytes()). It's not a char position in the text
    for (n, (start, idx)) in matches.into_iter().enumerate() {
        assert_eq!(test_data.get(n), Some(&(idx, start)));
        println!("Found '{}' at position {}", patterns.get(idx).unwrap_or(&"None"), start);
    }
}
