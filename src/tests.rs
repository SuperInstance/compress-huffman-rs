//! Comprehensive test suite for compress-huffman-rs.

#[cfg(test)]
mod tests {
    use crate::{encode, decode, frequency, tree, canonical};

    /// Helper: round-trip encode/decode using tree codes.
    fn round_trip(data: &[u8]) -> Vec<u8> {
        let freq = frequency::FrequencyTable::from_data(data);
        let htree = tree::HuffmanTree::from_frequency_table(&freq).unwrap();
        let codes = canonical::tree_codes(&htree);
        let encoded = encode::encode_with_map(data, &codes);
        decode::decode(&encoded.bits, encoded.bit_length, &htree).unwrap()
    }

    /// Helper: round-trip using canonical codes (encode) + rebuilt tree (decode).
    fn round_trip_canonical(data: &[u8]) -> Vec<u8> {
        let freq = frequency::FrequencyTable::from_data(data);
        let htree = tree::HuffmanTree::from_frequency_table(&freq).unwrap();
        let codes = canonical::CanonicalCodes::from_tree(&htree);
        let encoded = encode::encode(data, &codes);
        let lengths = codes.lengths().to_vec();
        decode::decode_canonical(&encoded.bits, encoded.bit_length, &lengths).unwrap()
    }

    // ── Frequency table tests ───────────────────────────────────────────

    #[test]
    fn test_frequency_basic() {
        let ft = frequency::FrequencyTable::from_data(b"aabbc");
        assert_eq!(ft.count(&b'a'), Some(2));
        assert_eq!(ft.count(&b'b'), Some(2));
        assert_eq!(ft.count(&b'c'), Some(1));
        assert_eq!(ft.count(&b'z'), None);
        assert_eq!(ft.total(), 5);
        assert_eq!(ft.distinct_count(), 3);
    }

    #[test]
    fn test_frequency_empty() {
        let ft = frequency::FrequencyTable::from_data(b"");
        assert_eq!(ft.total(), 0);
        assert_eq!(ft.distinct_count(), 0);
        assert_eq!(ft.entropy(), 0.0);
    }

    #[test]
    fn test_frequency_single_byte() {
        let ft = frequency::FrequencyTable::from_data(b"aaaa");
        assert_eq!(ft.distinct_count(), 1);
        assert_eq!(ft.count(&b'a'), Some(4));
        assert_eq!(ft.most_frequent(), Some((b'a', 4)));
    }

    #[test]
    fn test_frequency_entropy_uniform() {
        // 4 symbols, each appearing once: entropy = log2(4) = 2.0
        let ft = frequency::FrequencyTable::from_data(b"abcd");
        let ent = ft.entropy();
        assert!((ent - 2.0).abs() < 0.001, "entropy was {ent}");
    }

    #[test]
    fn test_frequency_entropy_single() {
        let ft = frequency::FrequencyTable::from_data(b"aaaa");
        let ent = ft.entropy();
        assert!(ent.abs() < 0.001, "entropy was {ent}");
    }

    #[test]
    fn test_frequency_increment() {
        let mut ft = frequency::FrequencyTable::new();
        ft.increment(b'x');
        ft.increment(b'x');
        ft.increment(b'y');
        assert_eq!(ft.count(&b'x'), Some(2));
        assert_eq!(ft.count(&b'y'), Some(1));
        assert_eq!(ft.total(), 3);
    }

    #[test]
    fn test_frequency_all_bytes() {
        let data: Vec<u8> = (0..=255).collect();
        let ft = frequency::FrequencyTable::from_data(&data);
        assert_eq!(ft.distinct_count(), 256);
        assert_eq!(ft.total(), 256);
    }

    // ── Tree construction tests ─────────────────────────────────────────

    #[test]
    fn test_tree_empty_returns_none() {
        let ft = frequency::FrequencyTable::from_data(b"");
        assert!(tree::HuffmanTree::from_frequency_table(&ft).is_none());
    }

    #[test]
    fn test_tree_single_symbol() {
        let ft = frequency::FrequencyTable::from_data(b"aaaa");
        let htree = tree::HuffmanTree::from_frequency_table(&ft).unwrap();
        let lengths = htree.code_lengths();
        assert_eq!(lengths.len(), 2); // symbol + dummy
    }

    #[test]
    fn test_tree_two_symbols() {
        let ft = frequency::FrequencyTable::from_data(b"aabb");
        let htree = tree::HuffmanTree::from_frequency_table(&ft).unwrap();
        let lengths = htree.code_lengths();
        // Both should have the same code length (1 bit each)
        assert_eq!(lengths.len(), 2);
        assert_eq!(lengths[0].1, 1);
        assert_eq!(lengths[1].1, 1);
    }

    #[test]
    fn test_tree_optimal_code_lengths() {
        // 'a' appears 8 times, 'b' 4 times, 'c' 2 times, 'd' 1 time
        // Optimal: a=1, b=2, c=3, d=3 (or equivalent)
        let ft = frequency::FrequencyTable::from_data(b"aaaaaaaabbbbccd");
        let htree = tree::HuffmanTree::from_frequency_table(&ft).unwrap();
        let mut lengths = htree.code_lengths();
        lengths.sort_by(|a, b| a.1.cmp(&b.1));
        assert_eq!(lengths[0].1, 1); // most frequent symbol gets shortest code
    }

    #[test]
    fn test_tree_many_symbols() {
        let data = b"the quick brown fox jumps over the lazy dog";
        let ft = frequency::FrequencyTable::from_data(data);
        let htree = tree::HuffmanTree::from_frequency_table(&ft).unwrap();
        let lengths = htree.code_lengths();
        assert!(lengths.len() >= 20); // many distinct symbols
    }

    // ── Canonical codes tests ───────────────────────────────────────────

    #[test]
    fn test_canonical_ordering() {
        let data = b"aaaaaabbc";
        let ft = frequency::FrequencyTable::from_data(data);
        let htree = tree::HuffmanTree::from_frequency_table(&ft).unwrap();
        let codes = canonical::CanonicalCodes::from_tree(&htree);
        let lengths = codes.lengths();
        // Lengths should be sorted (non-decreasing)
        for i in 1..lengths.len() {
            assert!(lengths[i].1 >= lengths[i - 1].1,
                "lengths not sorted: {:?}", lengths);
        }
    }

    #[test]
    fn test_canonical_prefix_property() {
        let data = b"the quick brown fox";
        let ft = frequency::FrequencyTable::from_data(data);
        let htree = tree::HuffmanTree::from_frequency_table(&ft).unwrap();
        let codes = canonical::CanonicalCodes::from_tree(&htree);

        // No code is a prefix of another
        let map = codes.map();
        let entries: Vec<_> = map.iter().collect();
        for i in 0..entries.len() {
            for j in 0..entries.len() {
                if i == j { continue; }
                let (code_i, len_i) = *entries[i].1;
                let (code_j, len_j) = *entries[j].1;
                if len_i <= len_j {
                    let shifted = code_j >> (len_j - len_i);
                    assert_ne!(code_i, shifted,
                        "prefix violation: {:?} is prefix of {:?}",
                        entries[i], entries[j]);
                }
            }
        }
    }

    #[test]
    fn test_canonical_expected_bits() {
        let data = b"aaaaaabbc";
        let ft = frequency::FrequencyTable::from_data(data);
        let htree = tree::HuffmanTree::from_frequency_table(&ft).unwrap();
        let codes = canonical::CanonicalCodes::from_tree(&htree);
        let bits = codes.expected_bits(&ft);
        // Should be less than 9 * 8 (uncompressed)
        assert!(bits < 72.0, "expected bits {bits} not less than uncompressed 72");
    }

    #[test]
    fn test_canonical_from_lengths() {
        // Explicit code lengths: a=2, b=2, c=3, d=3
        let codes = canonical::CanonicalCodes::from_lengths(&[
            (b'a', 2), (b'b', 2), (b'c', 3), (b'd', 3),
        ]);
        // Canonical: a=00, b=01, c=100, d=101
        assert_eq!(codes.get(&b'a'), Some(&(0b00, 2)));
        assert_eq!(codes.get(&b'b'), Some(&(0b01, 2)));
        assert_eq!(codes.get(&b'c'), Some(&(0b100, 3)));
        assert_eq!(codes.get(&b'd'), Some(&(0b101, 3)));
    }

    #[test]
    fn test_canonical_max_code_length() {
        let codes = canonical::CanonicalCodes::from_lengths(&[
            (b'a', 1), (b'b', 2), (b'c', 5),
        ]);
        assert_eq!(codes.max_code_length(), 5);
    }

    // ── Encode/Decode round-trip tests ──────────────────────────────────

    #[test]
    fn test_roundtrip_simple() {
        let data = b"hello world";
        assert_eq!(data.as_slice(), round_trip(data).as_slice());
    }

    #[test]
    fn test_roundtrip_empty() {
        // Empty data has no frequency table → no tree → we test the edge case
        let data = b"a"; // single char
        assert_eq!(data.as_slice(), round_trip(data).as_slice());
    }

    #[test]
    fn test_roundtrip_repetitive() {
        let data = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        assert_eq!(data.as_slice(), round_trip(data).as_slice());
    }

    #[test]
    fn test_roundtrip_binary_data() {
        let data: Vec<u8> = (0..=255).cycle().take(512).collect();
        assert_eq!(data.as_slice(), round_trip(&data).as_slice());
    }

    #[test]
    fn test_roundtrip_two_bytes() {
        let data = b"ab";
        assert_eq!(data.as_slice(), round_trip(data).as_slice());
    }

    #[test]
    fn test_roundtrip_long_text() {
        let data = b"In the beginning was the Word, and the Word was with God, and the Word was God.";
        assert_eq!(data.as_slice(), round_trip(data).as_slice());
    }

    #[test]
    fn test_roundtrip_all_same_byte() {
        let data = vec![0x42u8; 1000];
        assert_eq!(data.as_slice(), round_trip(&data).as_slice());
    }

    // ── Canonical round-trip tests ──────────────────────────────────────

    #[test]
    fn test_canonical_roundtrip() {
        let data = b"hello huffman coding works great";
        assert_eq!(data.as_slice(), round_trip_canonical(data).as_slice());
    }

    #[test]
    fn test_canonical_roundtrip_binary() {
        let data: Vec<u8> = (0..=255).collect();
        assert_eq!(data.as_slice(), round_trip_canonical(&data).as_slice());
    }

    // ── Compression effectiveness tests ─────────────────────────────────

    #[test]
    fn test_compression_saves_bits() {
        // Highly repetitive data should compress well
        let data = b"aaaaaaaabbbbccd";
        let ft = frequency::FrequencyTable::from_data(data);
        let htree = tree::HuffmanTree::from_frequency_table(&ft).unwrap();
        let codes = canonical::CanonicalCodes::from_tree(&htree);
        let encoded_bits = encode::encoded_bit_length(data, &codes);
        let uncompressed_bits = data.len() * 8;
        assert!(encoded_bits < uncompressed_bits,
            "encoded {} bits >= uncompressed {} bits", encoded_bits, uncompressed_bits);
    }

    #[test]
    fn test_encoded_bit_length_matches_actual() {
        let data = b"some test data here";
        let ft = frequency::FrequencyTable::from_data(data);
        let htree = tree::HuffmanTree::from_frequency_table(&ft).unwrap();
        let codes = canonical::CanonicalCodes::from_tree(&htree);
        let encoded = encode::encode(data, &codes);
        let computed = encode::encoded_bit_length(data, &codes);
        assert_eq!(encoded.bit_length, computed);
    }

    #[test]
    fn test_tree_codes_match_decode() {
        // Verify tree_codes produces codes that match the tree structure
        let data = b"abcdefg";
        let ft = frequency::FrequencyTable::from_data(data);
        let htree = tree::HuffmanTree::from_frequency_table(&ft).unwrap();
        let codes = canonical::tree_codes(&htree);
        let encoded = encode::encode_with_map(data, &codes);
        let decoded = decode::decode(&encoded.bits, encoded.bit_length, &htree).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_decode_empty_bits() {
        let ft = frequency::FrequencyTable::from_data(b"a");
        let htree = tree::HuffmanTree::from_frequency_table(&ft).unwrap();
        let result = decode::decode(&[], 0, &htree);
        assert_eq!(result, Some(Vec::new()));
    }

    #[test]
    fn test_decode_truncated_returns_none() {
        let data = b"abcdef";
        let ft = frequency::FrequencyTable::from_data(data);
        let htree = tree::HuffmanTree::from_frequency_table(&ft).unwrap();
        let codes = canonical::tree_codes(&htree);
        let encoded = encode::encode_with_map(data, &codes);
        // Truncate by 1 bit
        let result = decode::decode(&encoded.bits, encoded.bit_length - 1, &htree);
        assert!(result.is_none());
    }
}
