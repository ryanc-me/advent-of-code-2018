use std::error::Error;
use std::fs::File;
use std::io::{BufReader, prelude::*};
use std::collections::HashSet;

const FREQUENCIES_FILE: &str = "./input/frequencies.txt";

fn main() -> Result<(), Box<dyn Error>> {
    let freqs_buf = BufReader::new(File::open(&FREQUENCIES_FILE)?);
    let freqs: Vec<i32> = freqs_buf.lines()
        .filter_map(|x| x.ok())
        .map(|x| x.parse::<i32>().ok())
        .flatten()
        .collect();

    // part 1
    let total: i32 = freqs.iter().sum();
    println!("Total: {:?}", total);
    
    // part 2
    let mut seen: HashSet<i32> = HashSet::new();
    let mut freq: i32 = 0;
    let mut calibration: Option<i32> = None;
    for x in freqs.iter().cycle() {
        freq = freq + x;
        if seen.contains(&freq) {
            calibration = Some(freq);
            break
        }
        else {
            seen.insert(freq);
        }
    }
    println!("Calibration: {:?}", calibration);
 
    Ok(())
}