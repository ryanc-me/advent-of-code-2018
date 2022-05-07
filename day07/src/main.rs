#![feature(map_first_last)]
#[macro_use]
extern crate lazy_static;

use std::error;
use std::fs;
use std::io::{self, BufRead};
use std::result;
use std::str::FromStr;
use std::collections::{HashSet, HashMap, BTreeSet};
use regex::Regex;

const INPUT_FILE: &str = "./input/steps.txt";

type Result<T> = result::Result<T, Box<dyn error::Error>>;

fn main() -> Result<()> {
    let buf = io::BufReader::new(fs::File::open(&INPUT_FILE)?);
    let mut steps = buf
        .lines()
        .flatten()
        .map(|x| x.parse())
        .collect::<Result<Vec<_>>>()?;

    let p1 = part1(&mut steps)?;
    // let p2 = part2()?;

    println!("Part 1: {}", p1);
    // println!("Part 2: {}", p2);

    Ok(())
}

fn part1(steps: &mut Vec<Step>) -> Result<String> {
    // build a map of (step) -> [depends, on]
    let mut dependencies: HashMap<char, HashSet<char>> = HashMap::new();
    for step in steps.iter() {
        let entry = dependencies
            .entry(step.before)
            .or_insert(HashSet::new());
        entry.insert(step.name);
    }
    
    // find steps with no dependencies (initially available steps)
    let mut available: BTreeSet<char> = steps
        .iter()
        .filter(|x| !dependencies.contains_key(&x.name))
        .map(|x| x.name)
        .collect();

    // keep track of completed steps (in the correct order)
    let mut complete: Vec<char> = Vec::new();
    while let Some(current) = available.pop_first() {
        complete.push(current);
        for (name, deps) in &dependencies {
            if complete.contains(name) { continue }

            let mut ready = true;
            for dep in deps {
                if !complete.contains(dep) {
                    ready = false;
                }
            }
            if ready {
                available.insert(*name);
            }
        }
    }

    Ok(complete.into_iter().collect())
}

#[derive(Debug)]
struct Step {
    name: char,
    before: char,
}

impl FromStr for Step {
    type Err = Box<dyn error::Error>;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref REGEX: Regex = Regex::new(
                r"Step (?P<name>[\x00-\x7F]{1}) must be finished before step (?P<before>[\x00-\x7F]{1}) can begin\."
            ).unwrap();
        }
        let parts = REGEX
            .captures(s)
            .ok_or(format!("Malformed instruction line (could not match string against template): '{}'", s))?;

        // note: this `as char` cast might be dangerous. the regex is matching
        // exactly 1 character in the range 0x00-0x7F (0 - 127), so it *should*
        // be safe, but I haven't absolutely confirmed that.
        Ok(Step {
            name: parts["name"].as_bytes()[0] as char,
            before: parts["before"].as_bytes()[0] as char,
        })
    }
}
