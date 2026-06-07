//! Huffman tree building from frequency tables.

use crate::frequency::FrequencyTable;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// A node in the Huffman tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// Leaf node holding a byte symbol.
    Leaf { symbol: u8, weight: usize },
    /// Internal node with two children.
    Internal {
        weight: usize,
        left: Box<Node>,
        right: Box<Node>,
    },
}

impl Node {
    /// The weight (frequency sum) of this subtree.
    pub fn weight(&self) -> usize {
        match self {
            Node::Leaf { weight, .. } => *weight,
            Node::Internal { weight, .. } => *weight,
        }
    }

    /// If this is a leaf, return the symbol.
    pub fn symbol(&self) -> Option<u8> {
        match self {
            Node::Leaf { symbol, .. } => Some(*symbol),
            Node::Internal { .. } => None,
        }
    }

    /// Get the left child if internal.
    pub fn left(&self) -> Option<&Node> {
        match self {
            Node::Internal { left, .. } => Some(left),
            _ => None,
        }
    }

    /// Get the right child if internal.
    pub fn right(&self) -> Option<&Node> {
        match self {
            Node::Internal { right, .. } => Some(right),
            _ => None,
        }
    }
}

/// Wrapper for `BinaryHeap` ordering (min-heap via reversed `Ord`).
#[derive(Debug, Clone, Eq, PartialEq)]
struct HeapNode {
    node: Node,
}

impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse: smaller weight = higher priority
        other.node.weight().cmp(&self.node.weight())
    }
}

impl PartialOrd for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A Huffman tree that can be used for encoding and decoding.
#[derive(Debug, Clone)]
pub struct HuffmanTree {
    root: Node,
}

impl HuffmanTree {
    /// Build a Huffman tree from a frequency table.
    ///
    /// Returns `None` if the frequency table is empty.
    /// For a single symbol, a degenerate tree is produced.
    pub fn from_frequency_table(ft: &FrequencyTable) -> Option<Self> {
        if ft.total() == 0 {
            return None;
        }

        let mut heap: BinaryHeap<HeapNode> = BinaryHeap::new();

        for (&symbol, &count) in ft.iter() {
            heap.push(HeapNode {
                node: Node::Leaf {
                    symbol,
                    weight: count,
                },
            });
        }

        // If only one symbol, create a degenerate tree with one leaf
        if heap.len() == 1 {
            let only = heap.pop().unwrap().node;
            return Some(HuffmanTree {
                root: Node::Internal {
                    weight: only.weight(),
                    left: Box::new(only),
                    right: Box::new(Node::Leaf {
                        symbol: 0,
                        weight: 0,
                    }),
                },
            });
        }

        while heap.len() > 1 {
            let a = heap.pop().unwrap().node;
            let b = heap.pop().unwrap().node;
            let weight = a.weight() + b.weight();
            heap.push(HeapNode {
                node: Node::Internal {
                    weight,
                    left: Box::new(a),
                    right: Box::new(b),
                },
            });
        }

        let root = heap.pop().unwrap().node;
        Some(HuffmanTree { root })
    }

    /// Create a Huffman tree from a pre-built root node.
    pub fn from_root(root: Node) -> Self {
        HuffmanTree { root }
    }

    /// Get a reference to the root node.
    pub fn root(&self) -> &Node {
        &self.root
    }

    /// Walk the tree recursively to extract code lengths per symbol.
    ///
    /// Returns a vec of `(symbol, code_length)`.
    pub fn code_lengths(&self) -> Vec<(u8, u8)> {
        let mut lengths = Vec::new();
        Self::walk_lengths(&self.root, 0, &mut lengths);
        lengths
    }

    fn walk_lengths(node: &Node, depth: u8, lengths: &mut Vec<(u8, u8)>) {
        match node {
            Node::Leaf { .. } => {
                lengths.push((node.symbol().unwrap(), depth.max(1)));
            }
            Node::Internal { left, right, .. } => {
                Self::walk_lengths(left, depth + 1, lengths);
                Self::walk_lengths(right, depth + 1, lengths);
            }
        }
    }
}
