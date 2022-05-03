use std::error;
use std::fs;
use std::result;
use std::str::FromStr;
use std::io;


const INPUT_FILE: &str = "./input/claims.txt";

type Result<T> = result::Result<T, Box<dyn error::Error>>;


fn main() -> Result<()> {
    let buf = io::BufReader::new(fs::File::open(&INPUT_FILE)?);

    let claims = parse_claims(buf)?;

    let (p1, count_map) = part1(&claims)?;
    let p2 = part2(&claims, &count_map)?;

    println!("Overlapping Count: {}", p1);
    println!("Best Claim: {}", p2.id);

    Ok(())
}

fn parse_claims<R: io::BufRead>(buf: R) -> Result<Vec<Claim>> {
    let claims = buf.lines()
        .flatten()
        .map(|x| x.parse::<Claim>())
        .collect::<Result<Vec<Claim>>>();

    claims
}

fn part1(claims: &Vec<Claim>) -> Result<(u32, Vec<Vec<u8>>)> {
    let mut count_map = vec![vec![0u8; 1000]; 1000];
    for claim in claims {
        for x in claim.x..(claim.x+claim.w) {
            for y in claim.y..(claim.y+claim.h) {
                assert!(x <= 1000);
                assert!(y <= 1000);
                
                count_map[x as usize][y as usize] += 1;
            }
        }
    }
    
    let mut count_overlapping = 0u32;
    for row in &count_map {
        for cell in row {
            if *cell > 1 {
                count_overlapping += 1;
            }
        }
    }

    Ok((count_overlapping, count_map))
}

fn part2<'a, 'b>(claims: &'a Vec<Claim>, count_map: &'b Vec<Vec<u8>>) -> Result<&'a Claim> {
    'outer:
    for claim in claims {
        for x in claim.x..(claim.x+claim.w) {
            for y in claim.y..(claim.y+claim.h) {
                assert!(x <= 1000);
                assert!(y <= 1000);
                
                if count_map[x as usize][y as usize] > 1 {
                    continue 'outer;
                }
            }
        }

        // if we get to this point, then none of the cells in this
        // claim had double-ups!
        return Ok(claim)
    }

    Err("Could not find any non-overlapping claims! :-(".into())
}

#[derive(Debug)]
struct Claim {
    id: String,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl FromStr for Claim {
    type Err = Box<dyn error::Error>;

    fn from_str(s: &str) -> std::result::Result<Self, Box<dyn error::Error>> {
        fn parse<T>(part: Option<&str>, err: &'static str) -> Result<T>
        where T: std::str::FromStr, <T as FromStr>::Err: std::error::Error + 'static {
            Ok(part
                .ok_or(err)?
                .trim()
                .parse()?
            )
        }
        
        // treat these symbols as plain separators; this isn't as strict as
        // it (possibly) should be, but it does allow some level of inaccuracy
        // in the input data (e.g. spaces are stripped)
        let mut parts = s.split(&['@', ',', ':', 'x'][..]);
        let id = parse(parts.next(), "Could not find box id")?;
        let x = parse(parts.next(), "Could not find X position")?;
        let y = parse(parts.next(), "Could not find Y position")?;
        let w = parse(parts.next(), "Could not find width")?;
        let h = parse(parts.next(), "Could not find height")?;

        Ok(Self {
            id,
            x, y,
            w, h,
        })
    }
}
