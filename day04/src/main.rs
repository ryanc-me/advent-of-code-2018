use std::error;
use std::fs;
use std::result;
use std::io::{self, BufRead};
use std::collections::HashMap;
use std::cmp;

const INPUT_FILE: &str = "./input/guards.txt";

type Result<T> = result::Result<T, Box<dyn error::Error>>;

fn main() -> Result<()> {
    let buf = io::BufReader::new(fs::File::open(&INPUT_FILE)?);
    let guards = parse_guards(buf)?;

    let strat1 = part1(&guards)?;
    let strat2 = part2(&guards)?;

    println!("Strategy #1: {}", strat1);
    println!("Strategy #2: {}", strat2);

    Ok(())
}

fn parse_guards<T: BufRead>(buf: T) -> Result<Vec<Guard>> {
    let mut lines = buf
        .lines()
        .collect::<std::result::Result<Vec<_>, _>>()?;
    
    // sort entries by their date/time (thank you, ISO8601!)
    lines.sort();


    let mut lines_iter = lines.into_iter().peekable();
    let mut guards: HashMap<u32, Guard> = HashMap::new();
    while let Some(guard_line) = lines_iter.peek() {
        if !guard_line.contains("#") {
            // if the first few lines in a file are all time entries
            // (where we do not know the guard ID), then skip them
            continue
        }

        // extract the guard ID and date
        let guard_line = lines_iter.next().unwrap();
        let guard_id = guard_line
            .chars()
            .skip_while(|&x| x != '#')
            .skip(1) // skip the '#' char 
            .take_while(|&x| x.is_digit(10))
            .collect::<String>()
            .parse::<u32>()?;
        
        // note: the guards shift may actually start the day *before*
        // the entry lines (which are what we really care about),
        // so we'll attempt a peek. if the next line is actually
        // another guard shift line, we'll fall back to the current
        // line
        let shift_date: String;
        match lines_iter.peek() {
            Some(next_line) if !next_line.contains('#') => {
                shift_date = next_line[1..11].to_string();
            },
            _ => {
                shift_date = guard_line[1..11].to_string();
            }
        }

        // getsert the gaurd
        let guard = guards
            .entry(guard_id)
            .or_insert(Guard::new(guard_id));

        // creat a new shift for this chunk of lines
        let mut shift = Shift::new(shift_date);
        
        // loop through the sleep/wake lines
        let mut last_time = 0u8;
        let mut last_type = Consciousness::Awake;
        while let Some(entry_line) = lines_iter.peek() {
            if entry_line.contains('#') {
                // if our peek() returns a new line with a guard id,
                // then we should break this loop
                break
            }
            
            // extract data from the line
            let entry_line = lines_iter.next().unwrap();
            let entry_time = entry_line[15..17].parse()?;
            let entry_type = if entry_line.contains("falls asleep") {
                // note: we're actually matching the end of the *last entry*,
                // so the Asleep/Awake appear to be 'reversed' here
                Consciousness::Awake
            }
            else if entry_line.contains("wakes up") {
                Consciousness::Asleep
            }
            else {
                return Err(format!("Invalid entry line, should contain 'falls asleep' or 'wakes up': {}", &entry_line).into());
            };

            // note: we can skip adding the entry if the guard falls asleep
            // immediately (at 00:00)
            if entry_time > 0 {
                // build entry and push into list
                shift.push_entry(
                    last_time, 
                    entry_time - 1, 
                    entry_type, 
                    &entry_line
                );
            }

            // ensure the last_time is updated, so our next run through can
            // correctly assign a 'time_from'
            last_time = entry_time;
            last_type = entry_type;
        }

        // we need to push one last entry to account for a trailing
        // sleep/wake event
        shift.push_entry(
            last_time,
            60,
            match last_type {
                Consciousness::Awake => Consciousness::Asleep,
                Consciousness::Asleep => Consciousness::Awake,
            },
            "Finalizer line",
        );

        // finally, add the shift to the guard
        guard.push_shift(shift);
    }

    Ok(guards.into_values().collect())
}

fn part1(guards: &Vec<Guard>) -> Result<u32> {
    let mut guards = guards.clone();

    // find the sleepiest guard
    guards.sort_by_cached_key(|x| cmp::Reverse(x.time_asleep()));
    let guard = guards.get(0)
        .ok_or("There are no guards!".to_string())?;

    // build an array showing the minutes the guard is asleep
    let mut minutes = [0u8; 61];
    let asleep_entries = guard
        .get_entries()
        .filter(|x| x.consciousness == Consciousness::Asleep);
    for entry in asleep_entries {
        for i in entry.time_start..=entry.time_end {
            debug_assert!(i <= 60);
            minutes[i as usize] += 1;
        }
    }


    // sort and collect into a vec of (minute, num_sleeps)
    let mut minutes: Vec<(usize, u8)> = minutes
        .into_iter()
        .enumerate()
        .collect();
    minutes.sort_by_cached_key(|x| cmp::Reverse(x.1));

    debug_assert!(minutes.len() > 0);
    Ok(guard.id * minutes[0].0 as u32)
}

fn part2(guard: &Vec<Guard>) -> Result<u32> {
    // conver guards into a vec
    Ok(0)
}

#[derive(Debug, Clone)]
struct Guard {
    id: u32,
    shifts: Vec<Shift>
}

#[derive(Debug, Clone)]
struct Shift {
    date: String,
    entries: Vec<Entry>
}

#[derive(Debug, Clone)]
struct Entry {
    time_start: u8,
    time_end: u8,
    consciousness: Consciousness,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Consciousness {
    Asleep,
    Awake,
}

impl Guard {
    fn new(id: u32) -> Self {
        Self {
            id,
            shifts: Vec::new(),
        }
    }

    fn push_shift(&mut self, shift: Shift) {
        self.shifts.push(shift);
    }

    fn time_asleep(&self) -> u32 {
        self.shifts
            .iter()
            .map(|x| x.time_asleep())
            .sum()
    }

    fn get_entries(&self) -> impl Iterator<Item = &Entry> {
        self.shifts
            .iter()
            .map(|x| &x.entries)
            .flatten()
    }
}

impl Shift {
    fn new(date: String) -> Self {
        Self {
            date,
            entries: Vec::new(),
        }
    }

    fn push_entry(&mut self, start: u8, end: u8, consciousness: Consciousness, line: &str) {
        let entry = Entry::new(start, end, consciousness, line);
        self.entries.push(entry);
    }

    fn time_asleep(&self) -> u32 {
        self.entries
            .iter()
            .map(|x| x.time_asleep())
            .sum()
    }
}

impl Entry {
    fn new(start: u8, end: u8, consciousness: Consciousness, _line: &str) -> Self {
        Self {
            time_start: start,
            time_end: end,
            consciousness,
        }
    }

    fn time_asleep(&self) -> u32 {
        match self.consciousness {
            Consciousness::Asleep => {
                (self.time_end - self.time_start) as u32
            },
            Consciousness::Awake => {
                0
            }
        }
    }
}