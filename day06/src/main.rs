use std::error;
use std::fs;
use std::io::{self, BufRead};
use std::result;
use std::collections::{HashMap, HashSet};
use std::cmp;

const INPUT_FILE: &str = "./input/coordinates.txt";

type Result<T> = result::Result<T, Box<dyn error::Error>>;
type PointId = usize;
type GridEntry<'a> = (u32, Option<&'a Point>);

#[derive(Debug)]
struct Grid {
    points: Vec<Point>,
    xmin: i32,
    xmax: i32,
    ymin: i32,
    ymax: i32,
    w: i32,
    h: i32,
}

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
    id: PointId,
    size: u32,
    infinite: bool,
}

fn main() -> Result<()> {
    let buf = io::BufReader::new(fs::File::open(&INPUT_FILE)?);
    let mut grid = parse_grid(buf)?;

    let p1 = part1(&mut grid)?;
    let p2 = part2(&mut grid)?;

    println!("Part 1: {}", p1);
    println!("Part 2: {}", p2);

    Ok(())
}

fn parse_grid<B: BufRead>(buf: B) -> Result<Grid> {
    // convert the "x,y" lines into Points objects
    let points = buf
        .lines()
        .flatten()
        .enumerate()
        .map(|(id, x)| parse_line(id, &x))
        .collect::<Result<Vec<_>>>()?;

    if points.len() < 1 {
        return Err("Could not find any valid coordinates in the input".into());
    }

    let xmin = points.iter().map(|p| p.x).min().unwrap();
    let xmax = points.iter().map(|p| p.x).max().unwrap();
    let ymin = points.iter().map(|p| p.y).min().unwrap();
    let ymax = points.iter().map(|p| p.y).max().unwrap();
    let w = xmax - xmin + 1;
    let h = ymax - ymin + 1;

    let mut grid = Grid {
        points,
        xmin, xmax,
        ymin, ymax,
        w, h,
    };

    // if any point is closest to the border *around* our grid, then that
    // point is infinite
    let mut infinite_points: HashSet<PointId> = HashSet::new();
    for x in xmin-1..=xmax+1 {
        for y in ymin-1..=ymax+1 {
            if x > xmin && y > ymin && x < xmax && y < ymax { continue }
            if let Some(point) = closest_point(&grid, x, y).1 {
                infinite_points.insert(point.id);
            }
        }
    }
    for point in grid.points.iter_mut() {
        if infinite_points.contains(&point.id) {
            point.infinite = true;
        }
    }

    Ok(grid)
}

fn parse_line(id: PointId, line: &str) -> Result<Point> {
    let parts: Vec<&str> = line.split(",").collect();
    let x: i32 = parts[0].trim().parse()?;
    let y: i32 = parts[1].trim().parse()?;

    Ok(Point {
        id,
        x,
        y,
        size: 0,
        infinite: false,
    })
}

fn dist(ax: i32, ay: i32, bx: i32, by: i32) -> u32 {
    let dx = if ax > bx { ax - bx } else { bx - ax };
    let dy = if ay > by { ay - by } else { by - ay };

    (dx + dy) as u32
}

fn closest_point(grid: &Grid, x: i32, y: i32) -> GridEntry {
    let mut entry: GridEntry = (u32::MAX, None);
    for point in &grid.points {
        let d = dist(x, y, point.x, point.y);
        if d < entry.0 {
            entry = (d, Some(point));
        }
        else if d == entry.0 {
            entry = (d, None);
        }
    }

    entry
}

fn part1(grid: &mut Grid) -> Result<u32> {
    // find the closes 'point' for each position on the grid
    let mut point_owners: Vec<GridEntry> = vec![(u32::MAX, None); (grid.w * grid.h) as usize];
    for y in grid.ymin..=grid.ymax {
        for x in grid.xmin..=grid.xmax {
            let idx = ((x - grid.xmin) + grid.w * (y - grid.ymin)) as usize;
            point_owners[idx] = closest_point(grid, x, y);
        }
    }

    // map grid_id -> size
    let mut size_by_point = HashMap::new();
    point_owners
        .iter()
        .map(|x| x.1)
        .flatten()
        .for_each(|x| {
            let entry = size_by_point.entry(x.id).or_insert(0u32);
            *entry += 1;
        });
    
    // update all points with their computed size
    // for point in grid.points.iter_mut() {
    for point in grid.points.iter_mut().filter(|x| !x.infinite) {
        if let Some(size) = size_by_point.get(&point.id) {
            point.size = *size;
        }
    }

    // sort by size, desc
    grid.points.sort_by_key(|x| cmp::Reverse(x.size));
    Ok(grid.points[0].size)
}

fn part2(grid: &mut Grid) -> Result<u32> {
    let mut point_distances = vec![0; (grid.w * grid.h) as usize];
    for y in grid.ymin..=grid.ymax {
        for x in grid.xmin..=grid.xmax {
            let idx = ((x - grid.xmin) + grid.w * (y - grid.ymin)) as usize;
            let total_distance: u32 = grid.points
                .iter()
                .map(|p| dist(x, y, p.x, p.y))
                .sum();
            point_distances[idx] = total_distance;
        }
    }

    let area = point_distances
        .iter()
        .filter(|&x| *x < 10_000)
        .count();
    
    Ok(area as u32)
}
