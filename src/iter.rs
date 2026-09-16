use crate::{FastPatternMatcher, UType, Match, ALPHABET_SIZE};

pub struct MatchIterator<'a> {
    pub(crate) matcher: &'a FastPatternMatcher,
    pub(crate) bytes: &'a [u8],
    pub(crate) current_byte_idx: usize,
    pub(crate) current_node: usize,
}
impl<'a> Iterator for MatchIterator<'a> {
    type Item = Match;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {

        // Preload pointers/lengths to avoid repeated bounds checks in the loop
        let transitions = &self.matcher.transitions;
        let output_pattern_idx = &self.matcher.output_pattern_idx;
        let pattern_lengths = &self.matcher.pattern_lengths;

        // We start searching at current position
        while self.current_byte_idx < self.bytes.len() {
            let i = self.current_byte_idx;

            // SAFETY: The loop condition 'self.current_byte_idx < self.bytes.len()'
            // guarantees that the index is always within bounds of the bytes slice.
            let b = unsafe { *self.bytes.get_unchecked(i) as usize };

            // Go to the next node (DFA transition)
            // SAFETY: self.current_node is always a value from transitions, which are indices < num_nodes.
            // b is u8 cast to usize, so b < 256 (minimal ALPHABET_SIZE),
            // thus idx < num_nodes * 256 (minimal ALPHABET_SIZE).
            // usize::MAX < u32::MAX * (u8::MAX..u32::MAX)
            let idx = unsafe { self.current_node.unchecked_mul(ALPHABET_SIZE).unchecked_add(b) };
            unsafe { self.current_node = *transitions.get_unchecked(idx) as usize; }

                // SAFETY: self.current_node is a node index from transitions, so self.current_node < num_nodes.
                let p_idx = unsafe { *output_pattern_idx.get_unchecked(self.current_node) };
                if p_idx != UType::MAX {
                    let idx = p_idx as usize;

                    // SAFETY: idx is an index into pattern_lengths, which was filled during construction.
                    let len = unsafe { *pattern_lengths.get_unchecked(idx) };

                    // SAFETY: The variable i is the current index in the text.
                    // Since we only trigger a match when the last `len` characters of the processed text match the pattern,
                    // it is mathematically guaranteed that i + 1 >= len,
                    // ensuring the result is always a non-negative usize.
                    // i + 1 <= usize::MAX because i < self.bytes.len()
                    let pos = unsafe { i.unchecked_add(1).unchecked_sub(len) };
                    self.current_byte_idx = unsafe { i.unchecked_add(1) };

                    self.current_node = 0;
                    let m = Match { start_pos: pos, pattern_idx: idx };
                    return Some(m);
                }

                // SAFETY: self.current_byte_idx + 1 <= usize::MAX
                // because self.current_byte_idx < self.bytes.len()
                self.current_byte_idx = unsafe { self.current_byte_idx.unchecked_add(1) };
            }
        None
    }
}