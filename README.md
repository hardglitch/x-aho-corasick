# X-Aho-Corasick

An eXtra-performance, production-grade implementation of the **Aho-Corasick algorithm** in Rust. 

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
x-aho-corasick = { git = "https://github.com/hardglitch/x-aho-corasick.git" }
# or x-aho-corasick = "0.3.5" // planned
```

## Usage

### Find all occurrences
Use find_all_in when you need all non-overlapping occurrences of patterns in the text.

```rust
use x_aho_corasick::FastPatternMatcher;

fn main() {
    let patterns = vec!["apple", "app", "ple", "le"];
    let matcher = FastPatternMatcher::new(&patterns);
    let text = "applepie";

    let matches = matcher.find_all_in(text); // returns iterator
    for m in matches {
        println!("Found '{}' at position {}", patterns[m.pattern_index()], m.start_pos());
    }
}
```

### Find the first occurrence only
Use `find_any_in` for maximum performance when you only care if *at least one* of your patterns exists in the text. This method short-circuits as soon as a match is found.

```rust
use aho_corasick::FastPatternMatcher;

fn main() {
    let patterns = vec!["quick", "brown", "fox"];
    let matcher = FastPatternMatcher::new(&patterns);
    let text = "The quick brown fox jumps over the lazy dog";

    if let Some(m) = matcher.find_any_in(text) {
        println!("Found '{}' at position {}", patterns[m.pattern_index()], m.start_pos());
    }
}
```

## Limitations

- **Memory Usage**: Because it pre-calculates all transitions for every state in the Trie, memory usage can grow significantly if you have a very large number of states and many patterns.
- **Alphabet**: Optimized specifically for `u8` (byte) input.
