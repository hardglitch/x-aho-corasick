use x_aho_corasick::{FastPatternMatcher, Match};

fn main() {
    let patterns = ["ma", "mama", "cat", "at", "ма", "кошка", "лужа", "каша"];
    let matcher = FastPatternMatcher::new(&patterns);

    let text = "Мама (mama) вела кошку (cat) к луже, а в каше была (was) малина!".to_lowercase();
    let one_match = matcher.find_any_in(&text);
    let test_data = Some(Match::new(0, 4));

    // Start is a position in the byte array (text.as_bytes()). It's not a char position in the text
	assert_eq!(test_data, one_match);
    if let Some(m) = one_match {
        println!("Found '{}' at position {}", patterns[m.pattern_index()], m.start_pos());
    }
}