#![feature(rustc_private)]

extern crate arena;
extern crate binomial;

use arena::TypedArena;
use binomial::BinomialHeap;
use std::env;

pub fn main() {
    // Run like: ./binomial 6 5 4 3 2 1.
    let mut arena = TypedArena::new();
    let mut heap = BinomialHeap::new(&mut arena);
    for s in env::args().skip(1) {
        match s.parse::<i32>().ok() {
            Some(i) => {
                println!["pushing: {}", i];
                heap.push(i);
                // println!["head: {:?}", heap.peek()];
            },
            None => println!["can't parse arg: {}", s],
        }
    }
    println!["Size: {}", heap.len()];
}
