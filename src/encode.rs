//! Bit-level encoding using Huffman codes.

use crate::canonical::CanonicalCodes;
use crate::CodeMap;

/// Result of Huffman encoding: packed bits and the total bit count.
#[derive(Debug, Clone)]
pub struct Encoded {
    /// Packed bit data (MSB first within each byte).
    pub bits: Vec<u8>,
    /// Total number of valid bits in `bits`.
    pub bit_length: usize,
}

/// Encode a byte slice using canonical Huffman codes.
///
/// Bits are packed MSB-first into bytes. The final byte is zero-padded on
/// the right if `bit_length` is not a multiple of 8.
///
/// # Panics
///
/// Panics if a symbol in `data` is not present in the code map.
pub fn encode(data: &[u8], codes: &CanonicalCodes) -> Encoded {
    encode_with_map(data, codes.map())
}

/// Encode using a raw code map (for advanced usage).
pub fn encode_with_map(data: &[u8], map: &CodeMap) -> Encoded {
    let mut bits: Vec<u8> = Vec::new();
    let mut current: u64 = 0;
    let mut current_len: u8 = 0;
    let mut total_bits: usize = 0;

    for &symbol in data {
        let (code, len) = map[&symbol];
        current = (current << len) | code;
        current_len += len;

        while current_len >= 8 {
            current_len -= 8;
            bits.push(((current >> current_len) & 0xFF) as u8);
        }
        total_bits += len as usize;
    }

    if current_len > 0 {
        bits.push(((current << (8 - current_len)) & 0xFF) as u8);
    }

    Encoded {
        bits,
        bit_length: total_bits,
    }
}

/// Encode and return only the total bit length (no allocation for bits).
pub fn encoded_bit_length(data: &[u8], codes: &CanonicalCodes) -> usize {
    let mut total = 0;
    for &symbol in data {
        if let Some(&(_, len)) = codes.get(&symbol) {
            total += len as usize;
        }
    }
    total
}
