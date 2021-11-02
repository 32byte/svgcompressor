extern crate compressor;

use std::fs;
use compressor::*;

pub fn main() {
    // TODO: create a tui / arg parsing
    // TODO: better logging / statistics
    let content = fs::read_to_string("data/input.svg").expect("File not found!");

    let mut compressed = content;

    for i in 0..8 {
        println!("Iteration {}/8", i);
        compressed = compress(&compressed);
    }

    fs::write("data/output.svg", compressed).expect("Couldn't write to file!");
    println!("Done!");
}
