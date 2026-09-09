#![no_main]

use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    patterns: Vec<Vec<u8>>,
    text: Vec<u8>,
}

fuzz_target!(|input: FuzzInput| {
    let matcher = x_aho_corasick::FastPatternMatcher::new(&input.patterns);
    let _ = matcher.find_all_bytes_in(&input.text).collect::<Vec<_>>();
    let _ = matcher.find_all_bytes_overlapping_in(&input.text).collect::<Vec<_>>();
});