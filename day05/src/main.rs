use std::error;
use std::fs;
use std::result;
use std::io::{self, BufRead};

const INPUT_FILE: &str = "./input/polymer.txt";

type Result<T> = result::Result<T, Box<dyn error::Error>>;

fn main() -> Result<()> {
    let buf = io::BufReader::new(fs::File::open(&INPUT_FILE)?);

    let p1 = part1(buf)?;
    let p2 = part2(&p1)?;

    println!("Part 1: {}", p1.len());
    println!("Part 1: {}", p2.len());

    println!("\n\n");
    println!("Polymer #1:\n{}", p1);
    println!("Polymer #2:\n{}", p2);

    Ok(())
}

fn does_react(a: &char, b: &char) -> bool {
    // this is a bit ugly...
    (
        a.is_ascii_lowercase() && b.is_ascii_uppercase()
        ||
        a.is_ascii_uppercase() && b.is_ascii_lowercase()
    )
    && 
    a.to_ascii_uppercase() == b.to_ascii_uppercase()
}

fn reduce<I>(it: I) -> Result<String> where I: Iterator<Item = char> {
    let mut output: Vec<char> = Vec::new();
    let mut last_char: Option<char> = None;
    for c in it {
        // 1st case: current and last char match
        if last_char.is_some() && does_react(&c, &last_char.unwrap()) {
            last_char = output.pop();
            continue
        }
        // 2nd case: current doesn't match the previous or next
        // the `last_char` should be added to the output array
        if last_char.is_some() {
            output.push(last_char.unwrap());
        }
        last_char = Some(c);
    }
    // finally, we should push the last processed char
    if last_char.is_some() {
        output.push(last_char.unwrap());
    }

    Ok(output.into_iter().collect())
}

fn part1<B: BufRead>(buf: B) -> Result<String> {
    let iter = buf
        .bytes()
        .flatten()
        .map(|x| x as char);
        
    reduce(iter)
}

fn part2(polymer: &str) -> Result<String> {
    let mut min = usize::MAX;
    let mut new_polymer: Option<String> = None;
    for test_char in 65u8..=90u8 {
        let test_char = test_char as char;
        let iter = polymer
            .chars()
            .filter(|x| x.to_ascii_uppercase() != test_char);
        
        let res = reduce(iter)?;
        if res.len() <= min {
            min = res.len();
            new_polymer = Some(res);
        }
    }
    
    new_polymer.ok_or("No polymers found!".into())
}