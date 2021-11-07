extern crate compressor;

use clap::{App, Arg};
use compressor::*;
use log::{Level, LevelFilter};
use std::fs;

pub fn main() {
    let matches = App::new("svgcompressor")
        .version("0.1.0")
        .author("Alexey Lebedenko")
        .about("Merges neighboring rectangles with the same attributes into one")
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
                .help("Amount of iterations to compress")
                .default_value("8"),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .takes_value(false)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    match matches.occurrences_of("v") {
        0 => log::set_max_level(LevelFilter::Off),
        1 => log::set_max_level(LevelFilter::Info),
        _ => log::set_max_level(LevelFilter::Debug),
    }

    let iterations = match matches.value_of("iterations").unwrap().parse::<usize>() {
        Ok(iter) => iter,
        Err(_) => panic!("Interations have to be a positive natural number!"),
    };

    let content = fs::read_to_string(matches.value_of("INPUT").unwrap()).expect("File not found!");

    let mut compressed = content;
    for i in 0..iterations {
        if log::max_level() >= Level::Info {
            println!("Iteration {}/{}", i + 1, iterations);
        }
        compressed = compress(&compressed);
    }

    fs::write(matches.value_of("OUTPUT").unwrap(), compressed).expect("Couldn't write to file!");
    println!("Finished!");
}
