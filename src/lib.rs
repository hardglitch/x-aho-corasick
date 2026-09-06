use std::collections::VecDeque;

const ALPHABET_SIZE: usize = 256; // >=2^8, e.g 2^9, 2^10 etc

/// Production-grade Aho-Corasick implementation.
/// Optimized for high-speed scanning of large texts with massive pattern sets,
/// but with some memory overhead
pub struct FastPatternMatcher {
    /// Flattened transition table: [node_index * ALPHABET_SIZE + byte] -> next_node_index
    transitions: Vec<u32>,
    /// Dictionary links: points to the nearest node that is an end-of-pattern.
    dict_links: Vec<u32>,
    /// Stores the index of the pattern ending at this node (if any).
    output_pattern_idx: Vec<i32>, // -1 if no pattern ends here
    /// Pre-calculated lengths of patterns.
    pattern_lengths: Vec<usize>,
}

impl FastPatternMatcher {
    pub fn new(patterns: &[&str]) -> Self {
        let mut trie_nodes = vec![[0u32; ALPHABET_SIZE]]; // Root node
        let mut output_pattern_idx = vec![-1i32];
        let mut pattern_lengths = Vec::with_capacity(patterns.len());

        // --- Phase 1: Build Trie ---
        for (idx, pattern) in patterns.iter().enumerate() {
            let bytes = pattern.as_bytes();
            pattern_lengths.push(bytes.len());
            let mut curr = 0;

            for &b in bytes {
                let b = b as usize;
                if  trie_nodes[curr][b] == 0 {
                    trie_nodes[curr][b] = trie_nodes.len() as u32;
                    trie_nodes.push([0u32; ALPHABET_SIZE]);
                    output_pattern_idx.push(-1);
                }
                curr = trie_nodes[curr][b] as usize;
            }

            // Store pattern index at the terminal node
            output_pattern_idx[curr] = idx as i32;
        }

        let num_nodes = trie_nodes.len();
        let mut fail = vec![0u32; num_nodes];
        let mut dict_links = vec![0u32; num_nodes];
        let mut transitions = vec![0u32; num_nodes * ALPHABET_SIZE];

        // --- Phase 2: Build Failure and Dictionary Links (BFS) ---
        let mut queue = VecDeque::<u32>::new();

        for b in 0..ALPHABET_SIZE {
            let child = trie_nodes[0][b];
            if child > 0 {
                transitions[b] = child;
                queue.push_back(child);
            }
        }

        while let Some(u) = queue.pop_front() {
            let u_idx = u as usize;
            for (b, &v) in trie_nodes[u_idx].iter().enumerate() {
                let v_idx = v as usize;
                if v > 0 {
                    let f = fail[u_idx] as usize;
                    let idx = f * ALPHABET_SIZE + b;
                    fail[v_idx] = transitions[idx];

                    let f_link = fail[v_idx];
                    dict_links[v_idx] =
                        if output_pattern_idx[f_link as usize] != -1 { f_link }
                        else { dict_links[f_link as usize] };

                    let idx = u_idx * ALPHABET_SIZE + b;
                    transitions[idx] = v;
                    queue.push_back(v);
                }

                else {
                    let f = fail[u_idx] as usize;
                    // Automaton optimization: pre-calculate the transition for non-existent Trie edges
                    let idx1 = u_idx * ALPHABET_SIZE + b;
                    let idx2 = f * ALPHABET_SIZE + b;
                    transitions[idx1] = transitions[idx2];
                }
            }
        }

        FastPatternMatcher {
            transitions,
            dict_links,
            output_pattern_idx,
            pattern_lengths,
        }
    }

    #[inline]
    pub fn find_all_in(&self, text: &str) -> Vec<(usize, usize)> {
        self.find_inner(text, false)
    }
    #[inline]
    pub fn find_any_in(&self, text: &str) -> Vec<(usize, usize)> {
        self.find_inner(text, true)
    }
    #[inline(always)]
    fn find_inner(&self, text: &str, any: bool) -> Vec<(usize, usize)> {
        let bytes = text.as_bytes();
        let mut results = Vec::new();
        let mut curr = 0_usize;

        for (i, &b) in bytes.iter().enumerate() {
            let idx = curr * ALPHABET_SIZE + (b as usize);
            curr = self.transitions[idx] as usize;

            if curr != 0 {
                let mut temp = curr;
                while temp != 0 {
                    let p_idx = self.output_pattern_idx[temp];
                    if p_idx != -1 {
                        let idx = p_idx as usize;
                        let val = self.pattern_lengths[idx];
                        results.push((i + 1 - val, idx));
                        if any { return results }
                    }
                    // Jump to next pattern using dictionary links
                    temp = self.dict_links[temp] as usize;
                }
            }
        }
        results
    }
}