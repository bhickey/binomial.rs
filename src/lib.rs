#![feature(test)]

extern crate rand;
extern crate test;

use std::cmp;
use std::cmp::Ordering;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct BinomialHeap<T:Ord> {
    heads: VecDeque<Node<T>>,
}

#[derive(Debug)]
struct NodeData<T:Ord> {
    rank: u16,
    value: T,
    nodes: VecDeque<Node<T>>,
}

type Node<T> = Box<NodeData<T>>;

fn combine<T:Ord>(mut h1: Node<T>, h2: Node<T>) -> Node<T> {
    if h1.value >= h2.value {
        h1.rank += 1;
        h1.nodes.push_back(h2);
        h1
    } else {
        combine(h2, h1)
    }
}

// Destructively merges `a` and `b` into a new `VecDeque`.
fn merge_nodes<T:Ord>(a: &mut VecDeque<Node<T>>, b: &mut VecDeque<Node<T>>) -> VecDeque<Node<T>> {
    let mut result = VecDeque::with_capacity(cmp::max(a.len(), b.len()) + 1);
    while !a.is_empty() && !b.is_empty() {
        let a_rank = a[0].rank;
        let b_rank = b[0].rank;
        match a_rank.cmp(&b_rank) {
            Ordering::Less => result.push_back(a.pop_front().unwrap()),
            Ordering::Greater => result.push_back(b.pop_front().unwrap()),
            Ordering::Equal => {
                let a_node = a.pop_front().unwrap();
                let b_node = b.pop_front().unwrap();
                if a_node.value < b_node.value {
                    result.push_back(combine(a_node, b_node))
                } else {
                    result.push_back(combine(b_node, a_node))
                }
            }
        }
    }
    while !a.is_empty() {
        result.push_back(a.pop_front().unwrap());
    }
    while !b.is_empty() {
        result.push_back(b.pop_front().unwrap());
    }
    return result;
}

impl <T:Ord> BinomialHeap<T> {
    pub fn new() -> Self {
        BinomialHeap { heads: VecDeque::new() }
    }

    pub fn push(&mut self, value: T) {
        let mut v = VecDeque::with_capacity(1);
        v.push_back(Box::new(NodeData {
            rank: 0,
            value: value,
            nodes: VecDeque::new()
        }));
        self.heads = merge_nodes(
            &mut self.heads,
            &mut v);
    }

    pub fn pop(&mut self) -> Option<T> {
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

    pub fn peek(&self) -> Option<&T> {
        self.heads.iter().map(|n| &n.value).max()
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

    pub fn merge(&mut self, mut other: BinomialHeap<T>) {
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
        BinomialHeap::<usize>::new();
    }

    #[test]
    fn singleton_heap() {
        let mut t = BinomialHeap::new();
        assert_eq![t.len(), 0];
        assert![t.is_empty()];
        t.push(23i32);
        assert_eq![t.len(), 1];
        assert![!t.is_empty()];
        assert_eq![t.peek(), Some(&23i32)];
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
