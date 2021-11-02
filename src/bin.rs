extern crate compressor;

use clap::{App, Arg};
use compressor::*;
use std::fs;

pub fn main() {
    let matches = App::new("svgcompressor")
        .version("0.1.0")
        .author("Alexey Lebedenko")
        .about("Merges neighboring elements of the same kind into one")
        .arg(
            Arg::with_name("INPUT")
                .help("Input file to compress")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("Output file to save to")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("iterations")
                .short("i")
                .long("iter")
                .takes_value(true)
                .help("Ammount of iterations to compress")
                .default_value("8"),
        )
        .get_matches();

    let iterations = match matches.value_of("iterations").unwrap().parse::<usize>() {
        Ok(iter) => iter,
        Err(_) => panic!("Interations have to be a positive natural number!"),
    };

    let content = fs::read_to_string(matches.value_of("INPUT").unwrap()).expect("File not found!");

    let mut compressed = content;
    // TODO: better logging / statistics
    for i in 0..iterations {
        println!("Iteration {}/{}", i, iterations);
        compressed = compress(&compressed);
    }

    fs::write(matches.value_of("OUTPUT").unwrap(), compressed).expect("Couldn't write to file!");
    println!("Done!");
}
