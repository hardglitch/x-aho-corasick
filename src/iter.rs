use crate::{FastPatternMatcher, Match, ALPHABET_SIZE};
use crate::ring_buffer::RingBuffer;

pub struct MatchIterator<'a> {
    pub(crate) matcher: &'a FastPatternMatcher,
    pub(crate) bytes: &'a [u8],
    pub(crate) current_byte_idx: usize,
    pub(crate) current_node: usize,
	pub(crate) pending_matches: RingBuffer<Match>,
}
impl<'a> Iterator for MatchIterator<'a> {
    type Item = Match;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {

        // Preload pointers/lengths to avoid repeated bounds checks in the loop
        let transitions = &self.matcher.transitions;
        let dict_links = &self.matcher.dict_links;
        let output_pattern_idx = &self.matcher.output_pattern_idx;
        let pattern_lengths = &self.matcher.pattern_lengths;

		loop {
            // 1. If there are matches in the buffer, return the first one.
			if let Some(m) = self.pending_matches.pop_front() {
				return Some(m)
			}

            // 2. If the buffer is empty, search for a new byte in the text.
            if self.current_byte_idx >= self.bytes.len() {
                return None; // End of the text
            }

            let i = self.current_byte_idx;
            // Safety: The loop condition 'self.current_byte_idx < self.text_bytes.len()'
            // guarantees that the index is always within bounds of the text_bytes slice.
            let b = unsafe { *self.bytes.get_unchecked(i) as usize };

            // Go to the next node (DFA transition)
            // Safety: self.current_node is always a value from transitions, which are indices < num_nodes.
            // b is u8 cast to usize, so b < 256 (minimal ALPHABET_SIZE),
            // thus idx < num_nodes * 256 (minimal ALPHABET_SIZE).
            let idx = self.current_node * ALPHABET_SIZE + b;
            unsafe { self.current_node = *transitions.get_unchecked(idx) as usize; }

            let mut temp = self.current_node;
			let mut found_match = false;
			
            while temp > 0 {
                // Safety: temp is a node index from transitions, so temp < num_nodes.
                let p_idx = unsafe { *output_pattern_idx.get_unchecked(temp) };
                if p_idx != -1 {
                    let idx = p_idx as usize;

                    // Safety: idx is an index into pattern_lengths, which was filled during construction.
                    let len = unsafe { *pattern_lengths.get_unchecked(idx) };

                    // Safety: The variable i is the current index in the text.
                    // Since we only trigger a match when the last val characters of the processed text match the pattern,
                    // it is mathematically guaranteed that i + 1 >= val,
                    // ensuring the result is always a non-negative usize.
                    let pos = i + 1 - len;

					let m = Match::new(pos, idx);
					self.pending_matches.push_back(m);
					found_match = true;
				}

                // Jump to next pattern using dictionary links.
                // Safety: temp is always a valid node index (< num_nodes) because
                // all values in dict_links are either failure links or results of
                // previous dict_link lookups. The loop terminates because
                // dict_links always points to a node with a smaller depth.
                temp = unsafe { *dict_links.get_unchecked(temp) as usize };
            }

            self.current_byte_idx += 1;

			if found_match { continue; }
        }
    }
}