# X-Aho-Corasick

An eXtra-performance implementation of the **Aho-Corasick algorithm** in Rust.

## Features
- **Ultra-fast lookups**: Uses a full DFA (Deterministic Finite Automaton) for *O(n)* search time, where *n* is the length of the text.
- **Zero-copy matching**: Returns iterators over indices to minimize allocations.
 
## Performance Comparison
### Normal bench (search all)
*288 unique patterns (2..=30 bytes, multi-language) in 26.5 kb text (2871 words, multi-language)*

| Scenario | [aho-corasik](https://github.com/BurntSushi/aho-corasick) | [daachorse](https://github.com/daac-tools/daachorse) | `x-aho-corasick` |
| :--- | :---: | :---: | :---: |
| **Non-overlapping** | x1.00 | x1.30 | **x1.98x** |
| **Overlapping** | x1.00 | x1.33 | **x1.49x** |

### Hardcore bench (search all)
*561k unique patterns (2..=122 bytes, multi-language) in 95.4 Mb text (10_582_506 words, multi-language)*

| Scenario | [aho-corasik](https://github.com/BurntSushi/aho-corasick) | [daachorse](https://github.com/daac-tools/daachorse) | `x-aho-corasick` |
| :--- | :---: | :---: | :---: |
| **Non-overlapping** | x1.00 | x2.03 | **x2.36x** |
| **Overlapping** | x1.00 | **x1.67** | x1.34x |

## Coverage
- **cargo miri** - 100% clean
- **[cargo fuzz](https://github.com/rust-fuzz/cargo-fuzz)** - 100% clean

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
x-aho-corasick = "0.5.1"
# or x-aho-corasick = { git = "https://github.com/hardglitch/x-aho-corasick.git" }
```

## Basic usage

### Find all occurrences
Use `find_all_in` when you need all non-overlapping occurrences of patterns in the text.

```rust
use x_aho_corasick::FastPatternMatcher;

fn main() {
    let patterns = ["apple", "app", "ple", "le"];
    let matcher = FastPatternMatcher::new(&patterns);
    let text = "applepie";

    let matches = matcher.find_all_in(text); // returns iterator
    for m in matches {
        println!("Found '{}' at position {}", patterns[m.pattern()], m.start());
    }
}
```

If you need an overlapping then use overlapping version:

```rust
use x_aho_corasick::FastPatternMatcher;

fn main() {
    let patterns = ["apple", "app", "ple", "le"];
    let matcher = FastPatternMatcher::new(&patterns);
    let text = "applepie";

    let matches = matcher.find_all_overlapping_in(text); // returns iterator
    for m in matches {
        println!("Found '{}' at position {}", patterns[m.pattern()], m.start());
    }
}
```

### Find the first occurrence only
Use `find_any_in` for maximum performance when you only care if *at least one* of your patterns exists in the text. This method short-circuits as soon as a match is found.

```rust
use x_aho_corasick::FastPatternMatcher;

fn main() {
    let patterns = ["quick", "brown", "fox"];
    let matcher = FastPatternMatcher::new(&patterns);
    let text = "The quick brown fox jumps over the lazy dog";

    if let Some(m) = matcher.find_any_in(text) {
        println!("Found '{}' at position {}", patterns[m.pattern()], m.start());
    }
}
```

## Limitations

- **Memory Usage**: Because it pre-calculates all transitions for every state in the Trie, memory usage can grow significantly if you have a very large number of states and many patterns.
- **Alphabet**: Optimized specifically for `u8` (UTF-8) input.

## License
MIT
