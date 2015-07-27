#![feature(test)]
#![feature(rustc_private)]

extern crate arena;
extern crate rand;
extern crate test;

use arena::TypedArena;
use std::cmp;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::mem;

pub struct BinomialHeap<'a, T> where T: 'a + Ord {
    heads: VecDeque<Node<'a, T>>,
    allocator: &'a mut TypedArena<NodeData<'a, T>>,
    free_nodes: Vec<Node<'a, T>>,
}

#[derive(Debug)]
pub struct NodeData<'a, T> where T: 'a + Ord {
    rank: u16,
    value: Option<T>,
    nodes: VecDeque<Node<'a, T>>,
}

type Node<'a, T> = &'a mut NodeData<'a, T>;

// Combines h2 into the chain of nodes under h1.
// Precondition: h1.value >= h2.value.
fn combine<'a, T>(h1: &mut Node<'a, T>, h2: Node<'a, T>) where T: 'a + Ord {
    h1.rank += 1;
    h1.nodes.push_back(h2);
}

// Destructively merges `a` and `b` into a new `VecDeque`.
fn merge_nodes<'a, T>(a: &mut VecDeque<Node<'a, T>>, b: &mut VecDeque<Node<'a, T>>) -> VecDeque<Node<'a, T>> where T: 'a + Ord {
    let mut result = VecDeque::with_capacity(cmp::max(a.len(), b.len()) + 1);
    while !a.is_empty() && !b.is_empty() {
        let a_rank = a[0].rank;
        let b_rank = b[0].rank;
        match a_rank.cmp(&b_rank) {
            Ordering::Less => result.push_back(a.pop_front().unwrap()),
            Ordering::Greater => result.push_back(b.pop_front().unwrap()),
            Ordering::Equal => {
                let mut a_node = a.pop_front().unwrap();
                let mut b_node = b.pop_front().unwrap();
                if a_node.value >= b_node.value {
                    combine(&mut a_node, b_node);
                    result.push_back(a_node);
                } else {
                    combine(&mut b_node, a_node);
                    result.push_back(b_node);
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

impl <'a, T> BinomialHeap<'a, T> where T: 'a + Ord {
    fn new_node(&mut self, value: T) -> Node<'a, T> {
        match self.free_nodes.pop() {
            Some(n) => {
                n.rank = 0;
                n.value = Some(value);
                return n
            },
            None => unsafe {
                // The lifetime of the reference returned by TypedArena::alloc()
                // is pinned to the lifetime of the reference we have to the
                // arena itself. We know that the arena will be good for the
                // whole lifetime of 'a, but the borrow of self that we have for
                // the duration of new_node constrains our current borrow of
                // self.allocator to have a shorter lifetime. We are confident
                // that the NodeData allocated in the arena is good for all of
                // 'a, so we upcast the lifetime of the value that alloc()
                // returns.
                mem::transmute(self.allocator.alloc(NodeData { rank: 0,
                                                               value: Some(value),
                                                               nodes: VecDeque::with_capacity(1), }))
            },
        }
    }

    fn free_node(&mut self, n: Node<'a, T>) -> (T, VecDeque<Node<'a, T>>) {
        let value = mem::replace(&mut n.value, None).unwrap();
        let nodes = mem::replace(&mut n.nodes, VecDeque::with_capacity(1));
        self.free_nodes.push(n);
        return (value, nodes)
    }

    pub fn new(allocator: &'a mut TypedArena<NodeData<'a, T>>) -> Self {
        BinomialHeap { heads: VecDeque::new(),
                       allocator: allocator,
                       free_nodes: Vec::new(), }
    }

    pub fn push(&mut self, value: T) {
        let mut v = VecDeque::with_capacity(1);
        v.push_back(self.new_node(value));
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

        let mut n = self.heads.remove(min_idx).unwrap();
        let (value, mut nodes) =
            self.free_node(n);
        self.heads = merge_nodes(&mut self.heads, &mut nodes);
        return Some(value)
    }

    pub fn peek(&self) -> Option<&T> {
        self.heads.iter().flat_map(|n| n.value.as_ref()).max()
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

    pub fn merge(&mut self, mut other: BinomialHeap<'a, T>) {
      self.heads = merge_nodes(&mut self.heads, &mut other.heads);
    }
}

#[cfg(test)]
mod mytest {
    use ::BinomialHeap;
    use arena::TypedArena;
    use rand::{Rng, SeedableRng, StdRng};
    use std::clone::Clone;
    use std::collections::BinaryHeap;
    use test::Bencher;

    #[test]
    fn instantiate_empty_heap() {
        let mut arena = TypedArena::new();
        BinomialHeap::<usize>::new(&mut arena);
    }

    #[test]
    fn singleton_heap() {
        let mut arena = TypedArena::new();
        let mut t = BinomialHeap::new(&mut arena);
        assert_eq![t.len(), 0];
        assert![t.is_empty()];
        t.push(23i32);
        assert_eq![t.len(), 1];
        assert![!t.is_empty()];
        // assert_eq![t.peek(), Some(&23i32)];
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
            let mut arena = TypedArena::new();
            let mut heap = BinomialHeap::new(&mut arena);
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
