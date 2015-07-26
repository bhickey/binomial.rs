use std::cmp::Ordering;
use std::collections::VecDeque;
use std::env;
use std::fmt;
use std::result::Result;

#[derive(Clone,Debug)]
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

#[derive(Clone,Debug)]
struct NodeData {
    rank: u16,
    value: i32,
    nodes: VecDeque<Node>,
}

type Node = Box<NodeData>;

fn combine(mut h1: Node, h2: Node) -> Node {
    if h1.value <= h2.value {
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
                        println!["node ranks are equal"];
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
    fn new() -> Self {
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
            if node.value < self.heads[min_idx].value {
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
        self.heads.iter().map(|n| n.value).min()
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

fn main() {
    // Run like: ./binomial 6 5 4 3 2 1.
    let mut heap = BinomialHeap::new();
    for s in env::args().skip(1) {
        match s.parse::<i32>().ok() {
            Some(i) => {
                println!["pushing: {}", i];
                heap.push(i);
                println!["heap after: {}", heap];
                println!["head: {:?}", heap.peek()];
            },
            None => println!["can't parse arg: {}", s],
        }
    }
    println!["Size: {}", heap.len()];

}
