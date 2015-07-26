extern crate binomial;
use binomial::BinomialHeap;

use std::env;

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
