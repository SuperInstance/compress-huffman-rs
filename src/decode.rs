//! Bit-level decoding using Huffman trees.

use crate::tree::{HuffmanTree, Node};

/// Decode a bit stream using a Huffman tree.
///
/// Bits are read MSB-first from each byte. Only `bit_length` bits are
/// consumed; trailing padding in the last byte is ignored.
///
/// Returns `None` if the bit stream is corrupted (ends mid-symbol).
pub fn decode(bits: &[u8], bit_length: usize, tree: &HuffmanTree) -> Option<Vec<u8>> {
    if bit_length == 0 {
        return Some(Vec::new());
    }

    let mut result = Vec::new();
    let mut node = tree.root();
    let mut bits_read = 0;

    for &byte in bits {
        for i in (0..8).rev() {
            if bits_read >= bit_length {
                break;
            }
            let bit = (byte >> i) & 1;
            bits_read += 1;

            node = if bit == 0 {
                node.left()?
            } else {
                node.right()?
            };

            if let Some(symbol) = node.symbol() {
                result.push(symbol);
                node = tree.root();
            }
        }
    }

    // If we stopped partway through a symbol, that's corruption
    if !std::ptr::eq(node, tree.root()) {
        return None;
    }

    Some(result)
}

/// Decode assuming canonical ordering — rebuilds tree from code lengths.
pub fn decode_canonical(
    bits: &[u8],
    bit_length: usize,
    lengths: &[(u8, u8)],
) -> Option<Vec<u8>> {
    let tree = rebuild_tree_from_lengths(lengths)?;
    decode(bits, bit_length, &tree)
}

/// Rebuild a Huffman tree from canonical code lengths.
pub fn rebuild_tree_from_lengths(lengths: &[(u8, u8)]) -> Option<HuffmanTree> {
    if lengths.is_empty() {
        return None;
    }

    let mut sorted = lengths.to_vec();
    sorted.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

    let mut code: u64 = 0;
    let mut prev_len: u8 = 0;
    let mut code_entries: Vec<(u64, u8, u8)> = Vec::new(); // (code, length, symbol)

    for &(symbol, len) in &sorted {
        if prev_len > 0 {
            code += 1;
            code <<= len - prev_len;
        }
        code_entries.push((code, len, symbol));
        prev_len = len;
    }

    fn build_tree(codes: &[(u64, u8, u8)], depth: u8) -> Option<Node> {
        if codes.is_empty() {
            return None;
        }

        // If only one code left and it matches this depth, it's a leaf
        if codes.len() == 1 && codes[0].1 == depth {
            return Some(Node::Leaf {
                symbol: codes[0].2,
                weight: 0,
            });
        }

        type CodeEntry = (u64, u8, u8);
        let (mut lefts, mut rights): (Vec<CodeEntry>, Vec<CodeEntry>) =
            (Vec::new(), Vec::new());

        for &(c, l, s) in codes {
            if l == depth {
                // Leaf at exactly this depth but there are more codes — invalid canonical
                continue;
            }
            let shift = l - depth - 1;
            if (c >> shift) & 1 == 0 {
                lefts.push((c, l, s));
            } else {
                rights.push((c, l, s));
            }
        }

        let left = if lefts.is_empty() {
            Box::new(Node::Leaf { symbol: 0, weight: 0 })
        } else {
            Box::new(build_tree(&lefts, depth + 1)?)
        };

        let right = if rights.is_empty() {
            Box::new(Node::Leaf { symbol: 0, weight: 0 })
        } else {
            Box::new(build_tree(&rights, depth + 1)?)
        };

        Some(Node::Internal {
            weight: 0,
            left,
            right,
        })
    }

    let root = build_tree(&code_entries, 0)?;
    Some(HuffmanTree::from_root(root))
}
