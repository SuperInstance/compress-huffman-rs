//! Canonical Huffman code assignment and serialization helpers.

use crate::Code;
use crate::CodeMap;
use crate::tree::{HuffmanTree, Node};
use crate::FrequencyTable;

/// Canonical Huffman codes derived from a Huffman tree.
///
/// Canonical codes are computed from code lengths only, ensuring a
/// deterministic mapping regardless of the specific tree shape.
#[derive(Debug, Clone)]
pub struct CanonicalCodes {
    map: CodeMap,
    /// Sorted list of (symbol, code_length) for serialization.
    lengths: Vec<(u8, u8)>,
}

impl CanonicalCodes {
    /// Build canonical codes from a Huffman tree.
    ///
    /// Code lengths are extracted from the tree, then canonical codes are
    /// assigned in order of increasing length, breaking ties by symbol value.
    pub fn from_tree(tree: &HuffmanTree) -> Self {
        let mut lengths = tree.code_lengths();

        // Sort by (code_length, symbol)
        lengths.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

        let mut map = CodeMap::new();
        let mut code: u64 = 0;
        let mut prev_len: u8 = 0;

        for &(symbol, len) in &lengths {
            if prev_len > 0 {
                code += 1;
                code <<= len - prev_len;
            }
            map.insert(symbol, (code, len));
            prev_len = len;
        }

        CanonicalCodes { map, lengths }
    }

    /// Build canonical codes from explicit code lengths.
    ///
    /// The `lengths` slice contains `(symbol, code_length)` pairs.
    pub fn from_lengths(lengths: &[(u8, u8)]) -> Self {
        let mut sorted = lengths.to_vec();
        sorted.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

        let mut map = CodeMap::new();
        let mut code: u64 = 0;
        let mut prev_len: u8 = 0;

        for &(symbol, len) in &sorted {
            if prev_len > 0 {
                code += 1;
                code <<= len - prev_len;
            }
            map.insert(symbol, (code, len));
            prev_len = len;
        }

        CanonicalCodes { map, lengths: sorted }
    }

    /// Get the canonical code for a symbol.
    pub fn get(&self, symbol: &u8) -> Option<&Code> {
        self.map.get(symbol)
    }

    /// Access the full code map.
    pub fn map(&self) -> &CodeMap {
        &self.map
    }

    /// Access the sorted lengths vector.
    pub fn lengths(&self) -> &[(u8, u8)] {
        &self.lengths
    }

    /// Number of distinct symbols.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Whether the code map is empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Maximum code length among all symbols.
    pub fn max_code_length(&self) -> u8 {
        self.lengths.iter().map(|&(_, l)| l).max().unwrap_or(0)
    }

    /// Compute the total expected bits for a message with given frequencies.
    pub fn expected_bits(&self, freq: &FrequencyTable) -> f64 {
        let mut total = 0.0;
        for (&sym, &(.., len)) in &self.map {
            if let Some(count) = freq.count(&sym) {
                total += count as f64 * len as f64;
            }
        }
        total
    }
}

/// Build a code map directly from tree traversal (non-canonical).
///
/// This produces codes that match the tree's actual structure, suitable
/// for use with the original tree for decoding.
pub fn tree_codes(tree: &HuffmanTree) -> CodeMap {
    let mut map = CodeMap::new();
    walk_tree(tree.root(), 0, 0, &mut map);
    map
}

fn walk_tree(node: &Node, code: u64, len: u8, map: &mut CodeMap) {
    match node.symbol() {
        Some(symbol) => {
            map.insert(symbol, (code, len.max(1)));
        }
        None => {
            if let Some(left) = node.left() {
                walk_tree(left, code << 1, len + 1, map);
            }
            if let Some(right) = node.right() {
                walk_tree(right, (code << 1) | 1, len + 1, map);
            }
        }
    }
}
