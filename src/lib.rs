mod iter_ol;
mod iter;
mod ring_buffer;
#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use ring_buffer::RingBuffer;

/// Alphabet size must be at least 256 to cover all possible u8 values.
const ALPHABET_SIZE: usize = 256; // UTF-8

/// This one uses u32 for realistic tasks (to save memory),
/// but you can use other types
/// pattern_lengths < UType
/// pattern_length <= UType
type UType = u32; // Max pattern number = UType::MAX - 1

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Match {
    start_pos: UType,
    pattern_idx: UType,
}
impl Match {
    #[inline]
    pub fn new(start_pos: UType, pattern_idx: UType) -> Self {
        Self { start_pos, pattern_idx }
    }
    #[inline]
    pub fn start_pos(&self) -> usize {
        self.start_pos as usize
    }
    #[inline]
    pub fn pattern_index(&self) -> usize {
        self.pattern_idx as usize
    }
}

/// This Aho-Corasick implementation optimized for extreme-speed scanning
/// of large texts with massive pattern sets,
/// but at the cost of increased memory overhead (uses DFA).
#[derive(Debug)]
pub struct FastPatternMatcher {
    transitions: Vec<UType>,
    dict_links: Vec<UType>,
    output_pattern_idx: Vec<UType>,
    pattern_lengths: Vec<UType>,
}
impl FastPatternMatcher {
    pub fn new<T: AsRef<[u8]>>(patterns: &[T]) -> Self {
        let mut trie_nodes = vec![[0; ALPHABET_SIZE]]; // Root node
        let mut output_pattern_idx = vec![UType::MAX];
        let patterns_len = patterns.len().clamp(0, (UType::MAX - 1) as usize);
        let mut pattern_lengths = Vec::<UType>::with_capacity(patterns_len);

        // --- Phase 1: Build Trie ---
        for (idx, pattern) in patterns.iter().take(patterns_len).enumerate() {
            pattern_lengths.push(pattern.as_ref().len() as UType);
            let mut curr = 0;

            for &b in pattern.as_ref().iter() {
                let b = b as usize;
                // Safety: ALPHABET_SIZE >= 256, and b is an u8 cast to usize, so b < 256 (minimal ALPHABET_SIZE).
                unsafe {
                    if *trie_nodes.get_unchecked(curr).get_unchecked(b) == 0 {
                        *trie_nodes.get_unchecked_mut(curr).get_unchecked_mut(b) = trie_nodes.len() as UType;
                        trie_nodes.push([0; ALPHABET_SIZE]);
                        output_pattern_idx.push(UType::MAX);
                    }
                    curr = *trie_nodes.get_unchecked(curr).get_unchecked(b) as usize;
                }
            }

            // Store pattern index at the terminal node
            // Safety: curr is a valid index because it was just pushed or existed in trie_nodes
            unsafe { *output_pattern_idx.get_unchecked_mut(curr) = idx as UType; }
        }

        let num_nodes = trie_nodes.len();
        let mut fail = vec![0; num_nodes];
        let mut dict_links = vec![0; num_nodes];
        let mut transitions = vec![0; num_nodes * ALPHABET_SIZE];

        // --- Phase 2: Build Failure and Dictionary Links (BFS) ---
        let mut queue = VecDeque::<UType>::with_capacity(num_nodes);

        for b in 0..ALPHABET_SIZE {
            // Safety: root node is not empty and b in 0..<ALPHABET_SIZE always
            let child = unsafe { *trie_nodes.get_unchecked(0).get_unchecked(b) };
            if child > 0 {
                // Safety: transitions.len() >= ALPHABET_SIZE and b < 256 (minimal ALPHABET_SIZE).
                // Always within bounds.
                unsafe { *transitions.get_unchecked_mut(b) = child; }
                queue.push_back(child);
            }
        }

        while let Some(u) = queue.pop_front() {
            let u_idx = u as usize;
            // Safety: u_idx is an index of a node previously stored in trie_nodes.
            // Since all values added to the queue are valid indices from the trie,
            // u_idx will always be within [0, trie_nodes.len() - 1].
            let node = unsafe { *trie_nodes.get_unchecked(u_idx) };
            for (b, &v) in node.iter().enumerate() {
                let v_idx = v as usize;
                if v > 0 {
                    // Safety: u_idx is derived from values stored in trie_nodes.
                    // Since the number of nodes in the Trie defines the length of the fail array,
                    // any valid node index u will satisfy 0 <= u < fail.len().
                    let f = unsafe { *fail.get_unchecked(u_idx) as usize };
                    let idx = f * ALPHABET_SIZE + b;

                    // Safety: idx < num_nodes * ALPHABET_SIZE because u_idx < num_nodes
                    // and b < 256 (minimal ALPHABET_SIZE)
                    unsafe { *fail.get_unchecked_mut(v_idx) = *transitions.get_unchecked(idx); }

                    // Safety: v_idx is an index of a node stored in trie_nodes.
                    // Since the number of nodes in the Trie defines the length of the fail array,
                    // any valid node index v will satisfy 0 <= v < fail.len().
                    let f_link = unsafe { *fail.get_unchecked(v_idx) };

                    // Safety:
                    // 1. Bounds: f_link is a valid node index (0 <= f_link < num_nodes),
                    //    so access to output_pattern_idx and dict_links is safe.
                    // 2. Termination: Since f_link always points to a node with smaller depth,
                    //    the dictionary link chain is acyclic and will eventually reach the root (node 0).
                    unsafe {
                        *dict_links.get_unchecked_mut(v_idx) =
                            if *output_pattern_idx.get_unchecked(f_link as usize) != UType::MAX { f_link }
                            else { *dict_links.get_unchecked(f_link as usize) };
                    }

                    let idx = u_idx * ALPHABET_SIZE + b;

                    // Safety: Transitions optimization uses pre-calculated transitions from failure link
                    unsafe { *transitions.get_unchecked_mut(idx) = v; }
                    queue.push_back(v);
                }

                else {
                    // Safety: u_idx is derived from values stored in trie_nodes.
                    // Since the number of nodes in the Trie defines the length of the fail array,
                    // any valid node index u will satisfy 0 <= u < fail.len().
                    let f = unsafe { *fail.get_unchecked(u_idx) as usize };

                    // Automaton optimization: pre-calculate the transition for non-existent Trie edges
                    let idx1 = u_idx * ALPHABET_SIZE + b;
                    let idx2 = f * ALPHABET_SIZE + b;

                    // Safety: Transitions optimization uses pre-calculated transitions from failure link
                    unsafe { *transitions.get_unchecked_mut(idx1) = *transitions.get_unchecked(idx2); }
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
    pub fn find_all_in<'a>(&'a self, text: &'a str) -> iter::MatchIterator<'a> {
        self.find_all_bytes_in(text.as_bytes())
    }
    #[inline]
    pub fn find_all_overlapping_in<'a>(&'a self, text: &'a str) -> iter_ol::MatchIterator<'a> {
        self.find_all_bytes_overlapping_in(text.as_bytes())
    }
    #[inline]
    pub fn find_any_in<'a>(&'a self, text: &'a str) -> Option<Match> {
        self.find_all_bytes_in(text.as_bytes()).next()
    }

    #[inline]
    pub fn find_all_bytes_in<'a>(&'a self, bytes: &'a [u8]) -> iter::MatchIterator<'a> {
        iter::MatchIterator {
            matcher: self,
            bytes,
            current_byte_idx: 0,
            current_node: 0,
        }
    }
    #[inline]
    pub fn find_all_bytes_overlapping_in<'a>(&'a self, bytes: &'a [u8]) -> iter_ol::MatchIterator<'a> {
        iter_ol::MatchIterator {
            matcher: self,
            bytes,
            current_byte_idx: 0,
            current_node: 0,
            pending_matches: RingBuffer::new(),
        }
    }
    #[inline]
    pub fn find_any_byte_in<'a>(&'a self, bytes: &'a [u8]) -> Option<Match> {
        self.find_all_bytes_in(bytes).next()
    }
}