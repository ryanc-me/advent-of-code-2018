use std::error::Error;
use std::fs::File;
use std::io::{BufReader, prelude::*};

const FREQUENCIES_FILE: &str = "./input/frequencies.txt";

fn main() -> Result<(), Box<dyn Error>> {
    let freqs = BufReader::new(File::open(&FREQUENCIES_FILE)?);
    let result: i32 = freqs
        .lines()
        .filter_map(|x| x.ok())
        .map(|x| x.parse::<i32>().ok())
        .flatten()
        .sum();

    println!("{:?}", result);
    Ok(())
}
