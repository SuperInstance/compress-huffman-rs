//! Symbol frequency table construction and entropy computation.

use std::collections::HashMap;

/// A frequency table mapping byte symbols to their occurrence counts.
#[derive(Debug, Clone)]
pub struct FrequencyTable {
    counts: HashMap<u8, usize>,
    total: usize,
}

impl FrequencyTable {
    /// Build a frequency table from raw byte data.
    ///
    /// # Examples
    ///
    /// ```
    /// use compress_huffman_rs::frequency::FrequencyTable;
    ///
    /// let ft = FrequencyTable::from_data(b"aaaabbc");
    /// assert_eq!(ft.count(&b'a'), Some(4));
    /// assert_eq!(ft.count(&b'c'), Some(1));
    /// ```
    pub fn from_data(data: &[u8]) -> Self {
        let mut counts = HashMap::new();
        let mut total = 0;
        for &b in data {
            *counts.entry(b).or_insert(0) += 1;
            total += 1;
        }
        FrequencyTable { counts, total }
    }

    /// Create an empty frequency table.
    pub fn new() -> Self {
        FrequencyTable {
            counts: HashMap::new(),
            total: 0,
        }
    }

    /// Increment the count for a symbol.
    pub fn increment(&mut self, symbol: u8) {
        *self.counts.entry(symbol).or_insert(0) += 1;
        self.total += 1;
    }

    /// Get the count for a symbol (returns `None` if absent).
    pub fn count(&self, symbol: &u8) -> Option<usize> {
        self.counts.get(symbol).copied()
    }

    /// Total number of symbols recorded.
    pub fn total(&self) -> usize {
        self.total
    }

    /// Number of distinct symbols.
    pub fn distinct_count(&self) -> usize {
        self.counts.len()
    }

    /// Iterate over `(symbol, count)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&u8, &usize)> {
        self.counts.iter()
    }

    /// Compute the Shannon entropy of the distribution (in bits per symbol).
    ///
    /// Returns 0.0 for empty data.
    pub fn entropy(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        let total = self.total as f64;
        let mut ent = 0.0;
        for &count in self.counts.values() {
            if count == 0 {
                continue;
            }
            let p = count as f64 / total;
            ent -= p * p.log2();
        }
        ent
    }

    /// Return the most frequent symbol and its count.
    pub fn most_frequent(&self) -> Option<(u8, usize)> {
        self.counts
            .iter()
            .max_by_key(|&(_, c)| c)
            .map(|(&b, &c)| (b, c))
    }
}

impl Default for FrequencyTable {
    fn default() -> Self {
        Self::new()
    }
}
