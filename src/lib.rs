//! # compress-huffman-rs
//!
//! A pure-Rust Huffman coding compression library providing frequency analysis,
//! Huffman tree construction, canonical code generation, and round-trip
//! encode/decode.
//!
//! # Modules
//!
//! - [`frequency`] — Symbol frequency table construction and entropy computation.
//! - [`tree`] — Huffman tree building from frequency tables.
//! - [`canonical`] — Canonical Huffman code assignment and serialization helpers.
//! - [`encode`] — Bit-level encoding using Huffman codes.
//! - [`decode`] — Bit-level decoding using Huffman trees.
//!
//! # Quick Start
//!
//! ```
//! use compress_huffman_rs::{encode, decode, frequency, tree, canonical};
//!
//! let data = b"hello huffman";
//! let freq = frequency::FrequencyTable::from_data(data);
//! let htree = tree::HuffmanTree::from_frequency_table(&freq).unwrap();
//! // Use tree-derived codes for encoding (matches tree for decoding)
//! let codes = canonical::tree_codes(&htree);
//! let encoded = encode::encode_with_map(data, &codes);
//! let decoded = decode::decode(&encoded.bits, encoded.bit_length, &htree).unwrap();
//! assert_eq!(data.as_slice(), decoded.as_slice());
//! ```

use std::collections::HashMap;

/// A Huffman code represented as (bits_value, bit_length).
pub type Code = (u64, u8);

/// A mapping from byte symbols to their Huffman codes.
pub type CodeMap = HashMap<u8, Code>;

pub mod frequency;
pub mod tree;
pub mod canonical;
pub mod encode;
pub mod decode;

pub use frequency::FrequencyTable;
pub use tree::HuffmanTree;
pub use canonical::CanonicalCodes;

#[cfg(test)]
mod tests;
