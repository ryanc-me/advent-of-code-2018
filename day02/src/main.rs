use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom, prelude::*};
use std::collections::HashMap;

const BOX_NAMES_FILE: &str = "./input/boxes.txt";


fn main() -> Result<(), Box<dyn Error>> {
    let mut boxes_buf = BufReader::new(File::open(&BOX_NAMES_FILE)?);

    let checksum = part1(&mut boxes_buf)?;
    let box_id = part2(&mut boxes_buf)?;
    println!("Checksum: {}", checksum);
    println!("Box ID: {:?}", box_id);
    
    Ok(())
}

fn part1<R: BufRead + Seek>(buf: &mut R) -> Result<i32, Box<dyn Error>> {
    let mut boxes_iter = buf.lines()
        .filter_map(|x| x.ok())
        .peekable();
    let size_hint = boxes_iter.peek().unwrap().len();
    let mut seen: HashMap<char, u8> = HashMap::with_capacity(size_hint);

    let mut qty_2 = 0i32;
    let mut qty_3 = 0i32;
    for line in boxes_iter {
        for c in line.chars() {
            let entry = seen.entry(c).or_insert(0);
            *entry = *entry + 1;
        }
        let mut found_2 = false;
        let mut found_3 = false;
        for (_, v) in seen.drain() {
            if found_2 && found_3 {
                break;
            }
            if !found_2 && v == 2 {
                qty_2 = qty_2 + 1;
                found_2 = true;
            }
            else if !found_3 && v == 3 {
                qty_3 = qty_3 + 1;
                found_3 = true;
            }
        }
    }

    buf.seek(SeekFrom::Start(0))?;
    
    Ok(qty_2 * qty_3)
}

fn part2<R: BufRead + Seek>(buf: &mut R) -> Result<String, Box<dyn Error>> {
    let boxes: Vec<String> = buf.lines()
        .flatten()
        .collect();
    
    let iter1 = boxes.iter();
    let mut found: Option<(String, String)> = None;

    for (i, line1) in iter1.enumerate() {
        let iter2 = boxes.iter().skip(i);
        for line2 in iter2 {
            let uncommon = num_uncommon_chars(&line1, &line2);

            if uncommon == 1 {
                found = Some((line1.clone(), line2.clone()));
                break
            }
        }
    }

    if let Some(box_tuple) = found {
        let mut res = String::with_capacity(box_tuple.0.len());

        for (a, b) in box_tuple.0.chars().zip(box_tuple.1.chars()) {
            if a == b {
                res.push(a);
            }
        }
        Ok(res)
    }
    else {
        Err("Could not find any common box IDs".into())
    }
}

fn num_uncommon_chars(a: &str, b: &str) -> u32 {
    let iter_a = a.chars();
    let iter_b = b.chars();
    let mut uncommon = 0;

    for (char_a, char_b) in iter_a.zip(iter_b) {
        if char_a != char_b {
            uncommon += 1;
        }
    }

    uncommon
}
