#![feature(test)]

extern crate rand;
extern crate test;

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt;
use std::result::Result;

#[derive(Debug)]
pub struct BinomialHeap {
    heads: VecDeque<Node>,
}

fn format_node_list(nodes: &VecDeque<Node>, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    try![write![f, "["]];
    let mut i = nodes.iter();
    if let Some(n) = i.next() {
        try![write![f, "[{}", n.value]];
        if !n.nodes.is_empty() {
            try![write![f, " "]];
            try![format_node_list(&n.nodes, f)];
        }
        try![write![f, "]"]];
    }
    for n in i {
        try![write![f, ", [{}", n.value]];
        if !n.nodes.is_empty() {
            try![write![f, " "]];
            try![format_node_list(&n.nodes, f)];
        }
        try![write![f, "]"]];
    }
    write![f, "]"]
}

impl fmt::Display for BinomialHeap {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        format_node_list(&self.heads, f)
    }
}

#[derive(Debug)]
struct NodeData {
    rank: u16,
    value: i32,
    nodes: VecDeque<Node>,
}

type Node = Box<NodeData>;

fn combine(mut h1: Node, h2: Node) -> Node {
    if h1.value >= h2.value {
        h1.rank += 1;
        h1.nodes.push_back(h2);
        h1
    } else {
        combine(h2, h1)
    }
}

// Destructively merges `a` and `b` into a new `VecDeque`.
fn merge_nodes(a: &mut VecDeque<Node>, b: &mut VecDeque<Node>) -> VecDeque<Node> {
    let mut result = VecDeque::new();
    loop {
        match (a.pop_back(), b.pop_back()) {
            (None, None) => return result,
            (Some(h1), None) => result.push_back(h1),
            (None, Some(h2)) => result.push_back(h2),
            (Some(h1), Some(h2)) =>
                match h1.rank.cmp(&h2.rank) {
                    Ordering::Equal => {
                        let merged = combine(h1, h2);
                        let r = merged.rank;
                        if r != a.back().map(|n| n.rank).unwrap_or(0) {
                            if r != b.back().map(|n| n.rank).unwrap_or(0) {
                                let mut recur = merge_nodes(a, b);
                                loop {
                                    match recur.pop_back() {
                                        None => break,
                                        Some(x) => result.push_back(x),
                                    }
                                }
                                result.push_back(merged);
                            } else {
                                a.push_back(merged);
                                let mut recur = merge_nodes(a, b);
                                loop {
                                    match recur.pop_back() {
                                        None => break,
                                        Some(x) => result.push_back(x),
                                    }
                                }
                            }
                        } else {
                            if r != b.back().map(|n| n.rank).unwrap_or(0) {
                                b.push_back(merged);
                                let mut recur = merge_nodes(a, b);
                                loop {
                                    match recur.pop_back() {
                                        None => break,
                                        Some(x) => result.push_back(x),
                                    }
                                }
                            } else {
                                let mut recur = merge_nodes(a, b);
                                loop {
                                    match recur.pop_back() {
                                        None => break,
                                        Some(x) => result.push_back(x),
                                    }
                                }
                                result.push_back(merged);
                            }
                        }
                    },
                    Ordering::Less => {
                        b.push_back(h2);
                        let mut recur = merge_nodes(a, b);
                        loop {
                            match recur.pop_back() {
                                None => break,
                                Some(x) => result.push_back(x),
                            }
                        }
                        result.push_back(h1);
                    },
                    Ordering::Greater => {
                        a.push_back(h1);
                        let mut recur = merge_nodes(b, a);
                        loop {
                            match recur.pop_back() {
                                None => break,
                                Some(x) => result.push_back(x),
                            }
                        }
                        result.push_back(h2);
                    },
                },
        }
    }
}

impl BinomialHeap {
    pub fn new() -> Self {
        BinomialHeap { heads: VecDeque::new() }
    }

    pub fn push(&mut self, value: i32) {
        let mut v = VecDeque::new();
        v.push_back(Box::new(NodeData {
            rank: 0,
            value: value,
            nodes: VecDeque::new()
        }));
        self.heads = merge_nodes(
            &mut self.heads,
            &mut v);
    }

    pub fn pop(&mut self) -> Option<i32> {
        if self.heads.is_empty() {
            return None
        }
        let mut min_idx = 0usize;
        for (i, node) in self.heads.iter().enumerate() {
            if node.value > self.heads[min_idx].value {
                min_idx = i;
            }
        }
        let NodeData { value, mut nodes, .. } =
            *self.heads.remove(min_idx).unwrap();

        self.heads = merge_nodes(
            &mut self.heads,
            &mut nodes);

        return Some(value)
    }

    pub fn peek(&self) -> Option<i32> {
        self.heads.iter().map(|n| n.value).max()
    }

    pub fn is_empty(&self) -> bool {
        self.heads.is_empty()
    }

    pub fn len(&self) -> usize {
        let mut sz = 0;
        for node in self.heads.iter() {
            match node.rank {
                0 => sz += 1,
                x => sz += 2 << (x - 1),
            }
        }
        return sz
    }

    pub fn merge(&mut self, mut other: BinomialHeap) {
        self.heads = merge_nodes(&mut self.heads, &mut other.heads);
    }
}

#[cfg(test)]
mod mytest {
    use ::BinomialHeap;
    use std::clone::Clone;
    use std::collections::BinaryHeap;
    use ::test::Bencher;
    use rand::{Rng, SeedableRng, StdRng};

    #[test]
    fn instantiate_empty_heap() {
        BinomialHeap::new();
    }

    #[test]
    fn singleton_heap() {
        let mut t = BinomialHeap::new();
        assert_eq![t.len(), 0];
        assert![t.is_empty()];
        t.push(23i32);
        assert_eq![t.len(), 1];
        assert![!t.is_empty()];
        assert_eq![t.peek(), Some(23i32)];
        assert_eq![t.pop(), Some(23i32)];
    }

    fn get_values() -> Vec<i32> {
        let seed: &[_] = &[1, 2, 3, 4];
        let mut rng: StdRng = SeedableRng::from_seed(seed);
        (0..100).map(|_| rng.gen::<i32>()).collect()
    }

    #[bench]
    fn bench_binomial(b: &mut Bencher) {
        let values = get_values();
        let sorted = {
            let mut v = values.clone();
            v.sort_by(|x, y| y.cmp(x));
            v
        };

        b.iter(|| {
            let mut heap = BinomialHeap::new();
            for v in &values {
                heap.push(*v);
            }
            let mut heap_sorted = Vec::new();
            heap_sorted.reserve(heap.len());
            loop {
                match heap.pop() {
                    None => break,
                    Some(x) => heap_sorted.push(x),
                }
            }
            assert_eq![heap_sorted, sorted];
        });
    }

    #[bench]
    fn bench_builtin(b: &mut Bencher) {
        let values = get_values();
        let sorted = {
            let mut v = values.clone();
            v.sort_by(|x, y| y.cmp(x));
            v
        };
        b.iter(|| {
            let mut heap = BinaryHeap::new();
            for v in &values {
                heap.push(*v);
            }
            let mut heap_sorted = Vec::new();
            heap_sorted.reserve(heap.len());
            loop {
                match heap.pop() {
                    None => break,
                    Some(x) => heap_sorted.push(x),
                }
            }
            assert_eq![heap_sorted, sorted];
        });
    }
}
