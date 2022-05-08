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
const NUM_WORKERS: usize = 5;
const STEP_TIME: usize = 60;

type Result<T> = result::Result<T, Box<dyn error::Error>>;

fn main() -> Result<()> {
    let buf = io::BufReader::new(fs::File::open(&INPUT_FILE)?);
    let mut steps = buf
        .lines()
        .flatten()
        .map(|x| x.parse())
        .collect::<Result<Vec<_>>>()?;

    let p1 = part1(&mut steps)?;
    let p2 = part2(&mut steps)?;

    println!("Part 1: {}", p1);
    println!("Part 2: {}", p2.1);
    println!("     -> {}", p2.0);

    Ok(())
}

fn parse_deps(steps: &Vec<Step>) -> HashMap<char, HashSet<char>> {
    // parse the list of (step_name, step_dep) into a hash map
    // of (step) -> [list, of, dependencies]
    let mut dependencies: HashMap<char, HashSet<char>> = HashMap::new();
    for step in steps.iter() {
        let entry = dependencies
            .entry(step.before)
            .or_insert(HashSet::new());
        entry.insert(step.name);
    }

    dependencies
}

fn parse_initial(steps: &Vec<Step>, deps: &HashMap<char, HashSet<char>>) -> BTreeSet<char> {
    // find the initial steps (e.g. ones who aren't dependent on anything else)
    steps.iter()
        .filter(|x| !deps.contains_key(&x.name))
        .map(|x| x.name)
        .collect()
}

fn take_job(available: &mut BTreeSet<char>, running: &mut HashSet<char>) -> Option<(char, usize)> {
    // attempt to take a job from `available`; if successful,
    // return it, and update the set `running` with that job (char)
    match available.pop_first() {
        Some(step) => {
            let time = STEP_TIME + (step as u8 - 64) as usize;
            running.insert(step);
            Some((step, time))
        }
        _ => None
    }
}

fn part1(steps: &mut Vec<Step>) -> Result<String> {
    // build a map of (step) -> [depends, on]
    let dependencies = parse_deps(steps);
    
    // find steps with no dependencies (initially available steps)
    let mut available = parse_initial(steps, &dependencies);

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

fn part2(steps: &mut Vec<Step>) -> Result<(u32, String)> {
    let dependencies = parse_deps(steps);
    let mut available = parse_initial(steps, &dependencies);

    // track workers, whether they're working, and which char they're working on
    let mut workers: Vec<Option<(char, usize)>> = vec![None; NUM_WORKERS];

    // keep track of completed chars (in order), and currently running chars
    let mut complete: Vec<char> = Vec::new();
    let mut running: HashSet<char> = HashSet::new();

    // total time spent working, and the total number of jobs to do
    let mut total_time: u32 = 0;
    let total_todo = dependencies.keys().len() + available.len();


    loop {
        // attempt to do some work
        for maybe_worker in workers.iter_mut() {
            if let Some(ref mut worker) = maybe_worker {
                if worker.1 == 1 {
                    // the worker has a job assigned, and has completed it;
                    // grab a new one
                    running.remove(&worker.0);
                    complete.push(worker.0);
                    *maybe_worker = take_job(&mut available, &mut running);
                }
            }
            else {
                // worker is not working, try to grab a new job
                *maybe_worker = take_job(&mut available, &mut running);
            }

            if let Some(ref mut worker) = maybe_worker {
                // finally, do some work. note that elves are fast, and can
                // do work on the very second a job is assigned to them
                // (which is why this is run last; a job is added, then worked
                // on immediately)
                worker.1 -= 1;
            }
        }

        // update the list of available jobs
        for (name, deps) in &dependencies {
            if complete.contains(name) || running.contains(name) { continue }

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
        
        total_time += 1;
        if complete.len() == total_todo { break }
    }

    Ok((total_time, complete.into_iter().collect()))
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
