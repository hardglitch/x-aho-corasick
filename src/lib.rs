use std::arch::x86_64::{__m128i, __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi8, _mm_cmpeq_epi8, _mm_loadu_si128, _mm_movemask_epi8, _mm_set1_epi8};
use std::collections::VecDeque;

const ALPHABET_SIZE: usize = 256; // >=2^8. e.g 2^9, 2^10 etc

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum SIMDVersion {
    None = 0,
    AVX = 16,
    AVX2 = 32,
}

/// Aho-Corasick implementation.
/// Optimized for high-speed scanning of large texts with massive pattern sets.
pub struct FastPatternMatcher {
    /// Flattened transition table: [node_index * ALPHABET_SIZE + byte] -> next_node_index
    transitions: Vec<u32>,
    /// Dictionary links: points to the nearest node that is an end-of-pattern.
    dict_links: Vec<u32>,
    /// Stores the index of the pattern ending at this node (if any).
    output_pattern_idx: Vec<i32>, // -1 if no pattern ends here
    /// Pre-calculated lengths of patterns.
    pattern_lengths: Vec<usize>,
    /// We store a lookup table where we've pre-calculated which bytes are 'start' chars.
    /// To use AVX2 shuffle, we need to map byte values to indices.
    full_lookup_table: Vec<u8>,
}

impl FastPatternMatcher {
    pub fn new(patterns: &[&str]) -> Self {
        let mut trie_nodes = vec![[0u32; ALPHABET_SIZE]]; // Root node
        let mut output_pattern_idx = vec![-1i32];
        let mut pattern_lengths = Vec::with_capacity(patterns.len());
        let mut full_lookup_table = vec![0u8; ALPHABET_SIZE];

        // --- Phase 1: Build Trie ---
        for (idx, pattern) in patterns.iter().enumerate() {
            let bytes = pattern.as_bytes();
            if bytes.is_empty() { continue; }

            if let Some(&byte) = bytes.first() &&
               let Some(block) = full_lookup_table.get_mut(byte as usize)
            {
                *block = 1; // Mark start char for SIMD
            }
            pattern_lengths.push(bytes.len());

            let mut curr = 0;
            for &b in bytes {
                let b = b as usize;

                if trie_nodes.get(curr).is_some_and(|curr| curr.get(b).is_some_and(|&b| b == 0)) {

                    let len = trie_nodes.len() as u32;
                    if let Some(row) = trie_nodes.get_mut(curr) &&
                       let Some(element) = row.get_mut(b)
                    {
                        *element = len;
                    }

                    trie_nodes.push([0u32; ALPHABET_SIZE]);
                    output_pattern_idx.push(-1);
                }

                if let Some(row) = trie_nodes.get(curr) &&
                    let Some(&b) = row.get(b)
                {
                    curr = b as usize;
                }
            }

            // Store pattern index at the terminal node
            if let Some(idx_) = output_pattern_idx.get_mut(curr) {
                *idx_ = idx as i32;
            }
        }

        let num_nodes = trie_nodes.len();
        let mut fail = vec![0u32; num_nodes];
        let mut dict_links = vec![0u32; num_nodes];

        // Pre-allocate DFA transitions: num_nodes * ALPHABET_SIZE entries
        let mut dfa_transitions = vec![0u32; num_nodes.saturating_mul(ALPHABET_SIZE)];


        // --- Phase 2: Build Failure and Dictionary Links (BFS) ---
        let mut queue = VecDeque::<u32>::new();

        // Initialize level 1 nodes
        for b in 0..ALPHABET_SIZE {
            if let Some(&child) = trie_nodes.first().and_then(|row| row.get(b)) && child != 0 &&
               let Some(val) = dfa_transitions.get_mut(b)
            {
                *val = child;
                queue.push_back(child);
            }
        }

        while let Some(u) = queue.pop_front() {
            let u_idx = u as usize;
            for b in 0..ALPHABET_SIZE {
                if let Some(&v) = trie_nodes.get(u_idx).and_then(|row| row.get(b)) && v != 0 {
                    if let Some(&f) = fail.get(u_idx) {
                        let idx = (f as usize).saturating_mul(ALPHABET_SIZE).saturating_add(b);
                        if let Some(val1) = fail.get_mut(v as usize) &&
                           let Some(&val2) = dfa_transitions.get(idx)
                        {
                            *val1 = val2;
                        }
                    }

                    if let Some(&f_link) = fail.get(v as usize) &&
                       let Some(&dict_link) = dict_links.get(f_link as usize) &&
                       let Some(val) = dict_links.get_mut(v as usize) &&
                       let Some(&pat_idx) = output_pattern_idx.get(f_link as usize)
                    {
                        *val = if pat_idx != -1 { f_link } else { dict_link };
                    }

                    let idx = u_idx.saturating_mul(ALPHABET_SIZE).saturating_add(b);
                    if let Some(val) = dfa_transitions.get_mut(idx) { *val = v; }
                    queue.push_back(v);
                }

                else if let Some(&f) = fail.get(u_idx) {
                    // Automaton optimization: pre-calculate the transition for non-existent Trie edges
                    let idx1 = u_idx.saturating_mul(ALPHABET_SIZE).saturating_add(b);
                    let idx2 = (f as usize).saturating_mul(ALPHABET_SIZE).saturating_add(b);
                    if let Some(&val2) = dfa_transitions.get(idx2) &&
                       let Some(val1) = dfa_transitions.get_mut(idx1)
                    {
                        *val1 = val2;
                    }
                }
            }
        }

        FastPatternMatcher {
            transitions: dfa_transitions,
            dict_links,
            output_pattern_idx,
            pattern_lengths,
            full_lookup_table,
        }
    }

    /// Entry point for the search. Uses runtime feature detection to dispatch.
    pub fn find_all_in(&self, text: &str) -> Vec<(usize, usize)> {
        if is_x86_feature_detected!("avx2") {
            unsafe { self.find_all_avx2(text, false) }
        } else if is_x86_feature_detected!("avx") {
            unsafe { self.find_all_avx(text, false) }
        } else {
            self.find_all_scalar(text, false)
        }
    }
    pub fn find_any_in(&self, text: &str) -> Vec<(usize, usize)> {
        if is_x86_feature_detected!("avx2") {
            unsafe { self.find_all_avx2(text, true) }
        } else if is_x86_feature_detected!("avx") {
            unsafe { self.find_all_avx(text, true) }
        } else {
            self.find_all_scalar(text, true)
        }
    }

    /// AVX2 Implementation (32-byte skips)
    #[target_feature(enable = "avx2")]
    fn find_all_avx2(&self, text: &str, any: bool) -> Vec<(usize, usize)> {
        self.dispatch(text, |chunk, offset| self.search_core(chunk, offset, SIMDVersion::AVX2), any)
    }

    /// AVX1 Implementation (16-byte skips)
    #[target_feature(enable = "avx")]
    fn find_all_avx(&self, text: &str, any: bool) -> Vec<(usize, usize)> {
        self.dispatch(text, |chunk, offset| self.search_core(chunk, offset, SIMDVersion::AVX), any)
    }

    /// Scalar Implementation (No SIMD)
    fn find_all_scalar(&self, text: &str, any: bool) -> Vec<(usize, usize)> {
        self.dispatch(text, |chunk, offset| self.search_core(chunk, offset, SIMDVersion::None), any)
    }

    /// Internal dispatcher
    fn dispatch<F>(&self, text: &str, search_func: F, any: bool) -> Vec<(usize, usize)>
        where F: Fn(&[u8], usize) -> Vec<(usize, usize)>,
    {
        let bytes = text.as_bytes();
        if bytes.is_empty() { return Vec::new(); }

        const CHUNK_SIZE: usize = 128 * 1024;
        let max_pat_len = *self.pattern_lengths.iter().max().unwrap_or(&0);

        let take = if any { 1 } else { usize::MAX };
        bytes.chunks(CHUNK_SIZE)
            .enumerate()
            .flat_map(|(idx, chunk)| {
                let offset = idx.saturating_mul(CHUNK_SIZE);
                // To handle pattern straddling, we ensure the searcher sees enough context
                let end = std::cmp::min(offset.saturating_add(chunk.len()).saturating_add(max_pat_len), bytes.len());
                let slice = bytes.get(offset..end).unwrap_or_default();
                search_func(slice, offset)
            })
            .take(take)
            .collect()
    }

    #[inline(always)]
    fn search_core(&self, chunk: &[u8], offset: usize, simd_ver: SIMDVersion) -> Vec<(usize, usize)> {
        let mut results = Vec::new();
        let mut curr = 0usize;
        let mut i = 0;
        let step = simd_ver as usize;

        while i < chunk.len() {
            // Skip Logic
            if simd_ver != SIMDVersion::None && curr == 0 && i.saturating_add(step) <= chunk.len() {
                let skipped = self.skip(chunk, i, simd_ver);
                i = i.saturating_add(skipped);
            }

            if let Some(&b) = chunk.get(i) &&
               let Some(&curr_) = self.transitions.get(curr.saturating_mul(ALPHABET_SIZE).saturating_add(b as usize))
            {
                curr = curr_ as usize;
            }

            if curr != 0 {
                self.collect_matches(&mut results, curr, offset.saturating_add(i));
            }
            i = i.saturating_add(1);
        }
        results
    }

    #[inline(always)]
    fn collect_matches(&self, results: &mut Vec<(usize, usize)>, mut temp: usize, pos: usize) {
        while temp != 0 {
            if let Some(&p_idx) = self.output_pattern_idx.get(temp) && p_idx != -1 &&
               let Some(&p) = self.pattern_lengths.get(p_idx as usize)
            {
                let idx = p_idx as usize;
                let pos_ = pos.saturating_add(1).saturating_sub(p);
                results.push((pos_, idx));
            }

            if let Some(&temp_) = self.dict_links.get(temp) {
                temp = temp_ as usize;
            }
        }
    }

    #[inline(always)]
    fn skip(&self, bytes: &[u8], i: usize, simd_ver: SIMDVersion) -> usize {
        let mut current_i = i;
        let step = simd_ver as usize;

        while current_i.saturating_add(step) <= bytes.len() {
            let mut found = false;
            match simd_ver {
                SIMDVersion::AVX2 => unsafe {
                    let data = _mm256_loadu_si256(bytes.as_ptr().add(current_i) as *const __m256i);
                    for byte_idx in 0..ALPHABET_SIZE {
                        if let Some(&b) = self.full_lookup_table.get(byte_idx) && b == 1 {
                            let target = _mm256_set1_epi8(byte_idx as i8);
                            let cmp = _mm256_cmpeq_epi8(data, target);
                            if _mm256_movemask_epi8(cmp) != 0 {
                                found = true;
                                break;
                            }
                        }
                    }
                }
                SIMDVersion::AVX => unsafe {
                    let data = _mm_loadu_si128(bytes.as_ptr().add(current_i) as *const __m128i);
                    for byte_idx in 0..ALPHABET_SIZE {
                        if let Some(&b) = self.full_lookup_table.get(byte_idx) && b == 1 {
                            let target = _mm_set1_epi8(byte_idx as i8);
                            let cmp = _mm_cmpeq_epi8(data, target);
                            if _mm_movemask_epi8(cmp) != 0 {
                                found = true;
                                break;
                            }
                        }
                    }
                }
                _ => {}
            };
            if found { return current_i.saturating_sub(i); }
            current_i = current_i.saturating_add(step);
        }

        // Fallback to scalar for remaining or if mask was non-zero but no start char found
        let mut res = current_i;
        while res < bytes.len() && res < i.saturating_add(step) {
            if let Some(&b) = bytes.get(res) &&
               let Some(&b) = self.full_lookup_table.get(b as usize) && b == 1
            {
                break;
            }
            res = res.saturating_add(1);
        }
        res.saturating_sub(i)
    }
}
