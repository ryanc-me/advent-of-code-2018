#![feature(int_roundings)]

use std::error;
use std::fs;
use std::io::{self, BufRead};
use std::result;
use std::str::FromStr;

const INPUT_FILE: &str = "./input/license.txt";

type Result<T> = result::Result<T, Box<dyn error::Error>>;

fn main() -> Result<()> {
    let buf = io::BufReader::new(fs::File::open(&INPUT_FILE)?);
    let numbers = parse_numbers(buf)?;

    let (p1_stack, p2_stack, _root_node) = parse_nodes(&numbers)?;
    let p1_recursive = part1(&numbers);
    let p2_recursive = part2(&numbers);

    println!("Part 1 (Stack):     {}", p1_stack);
    println!("Part 1 (Recursive): {}", p1_recursive);
    println!("Part 2 (Stack):     {}", p2_stack);
    println!("Part 2 (Recursive): {}", p2_recursive);

    Ok(())
}

fn parse_numbers<B: BufRead>(buf: B) -> Result<Vec<usize>> {
    // this was an attempt to parse BufRead -> Vec<usize> directly, without
    // first allocating a string. however, split() is returning a Vec<usize>,
    // so I think this method will actually cause *more* allocations (since
    // we'll have result.len() allocations of Vec<u8>, versus a single
    // buf.to_string() and associated "doubling" allocations?)

    buf.split(' ' as u8)
        .flatten()
        .map(|vec| {
            // we can re-use the Vec<u8> here to build a string-repr of the numbers
            let s = String::from_utf8(vec)?;
            let n = u8::from_str(&s.trim());
            n.map_err(
                |x| format!("Failed to parse {}: {}", &s, x).into()
            )
            .map(|x| x as usize)
        })
        .collect()
}

fn part1(numbers: &[usize]) -> usize {
    fn recurse(n: &[usize]) -> (usize, usize) {
        match (n[0], n[1]) {
            (0, nmeta) => {
                // no children, all items are metadata.
                let metadata = n[2..nmeta+2].iter().sum();
                (nmeta + 2, metadata)
            },
            (nchild, nmeta) => {
                // the item has children
                let mut index = 2;
                let mut metadata = 0;

                // loop children and sum their metadata entries
                for _ in 0..nchild {
                    let (len, meta) = recurse(&n[index..]);
                    index += len;
                    metadata += meta;
                }

                // now, the starting index of this items' metadat entries is
                // known
                metadata += n[index..index+nmeta].iter().sum::<usize>();

                (index + nmeta, metadata)
            }
        }
    }

    let (_len, meta) = recurse(numbers);
    meta
}

fn part2(numbers: &[usize]) -> usize {
    fn recurse(n: &[usize]) -> (usize, usize) {
        match (n[0], n[1]) {
            (0, nmeta) => {
                let metadata = n[2..nmeta+2].iter().sum();
                (nmeta + 2, metadata)
            },
            (nchild, nmeta) => {
                let mut index = 2;
                let mut children: Vec<usize> = Vec::with_capacity(nchild);
                let mut meta = 0;

                // as before, loop children to grab their metadata entries
                // this time, we can build a vec to use for look-ups later
                for _ in 0..nchild {
                    let (len, meta) = recurse(&n[index..]);
                    index += len;
                    children.push(meta);
                }

                // loop through this items' metadata entries, and look-up
                // the corresponding childs' metadata
                let metadata = &n[index..index+nmeta];
                for i in metadata.iter().filter(|x| **x <= nchild) {
                    meta += children[*i - 1];
                }

                (index + nmeta, meta)
            }
        }
    }

    let (_len, meta) = recurse(numbers);
    meta
}

fn parse_nodes(numbers: &Vec<usize>) -> Result<(usize, usize, Node)> {
    // the approach is to loop through (nchild, nmeta, index) items from the input,
    // adding them to the stack.
    //
    // `index` is set initially to the index immediately after the items' nmeta
    // entry (e.g. '2' for the first item, '4' for the 2nd, etc - this will at first
    // correspond with the index of the current items' first child).
    //
    //
    // when looping, if an item with `nchild` == 0 is found, it can be processed
    // immediately, by:
    // - building a Node for it
    // - decrementing the parents' `nchild`
    // - updating the parents' `index`: `parent.index += nchild + nmeta`
    //
    // the parent node will be the 2nd last node in the stack
    //
    // updating the parents index effectively moves the index forward by "1 child",
    // so that when all children are processed, `index` points at the items' first
    // metadata entry
    //
    // once all children for an item have been processed, they will have been pop()-ed
    // from the stack, so that the next pop() will produce the node, with its `index`
    // set correctly to the 1st metadata entry. the Node can be created.
    //
    //
    // part1 can be found by summing metadata entries as they're found
    //
    // part2 can be found by tracking the `meta_sum` of each node.
    // - for nchild == 0, this will be sum(metadata)
    // - for nchild >= 1, this will be sum(child.meta_sum), conveniently we are
    //   already looping in the correct order (build nchild==0 Nodes first, then
    //   only process the parent once all children are done)

    let mut stack: Vec<Item> = Vec::new();
    let mut part1 = 0;
    
    // seed the stack
    let nchild = numbers[0];
    let nmeta = numbers[1];
    let item = Item::new(
        2,
        nchild,
        nmeta,
    );
    stack.push(item);

    while let Some(item) = stack.last_mut() {
        if item.nchild == 0 {
            // if this items' children are all complete, then its `index`
            // is set correctly, and we can create a Node for it
            let new_index = item.index+item.nmeta;
            let metadata = &numbers[item.index..new_index];
            part1 += metadata.iter().sum::<usize>();
            
            // the item is finished, so it can be pop()-ed from the stack
            // (note that the outer while loop is only grabbing last_mut().
            // the borrow checker is happy because it can drop the original
            // `item` ref before the pop() here.)
            match stack.pop() {
                Some(item) => {
                    let node = Node::new(metadata, item.children);
                    match stack.last_mut () {
                        Some(parent_item) => {
                            // if there is a parent item, then add the new Node to its children
                            parent_item.push_child(node, new_index);
                            continue
                        },
                        None => {
                            // else, this must be the root node!
                            return Ok((part1, node.meta_sum, node))
                        }
                    }
                },
                // this shouldn't be possible (we know the stack has at least 1 item)
                None => break
            }
        }
        else {
            // this item still has children; build one of them
            let nchild = numbers[item.index];
            let nmeta = numbers[item.index+1];
            if nchild == 0 {
                // case #1; this is a no-child node, so we can immediately
                // build a Node for it
                let new_index = item.index + 2 + nmeta;
                let metadata = &numbers[item.index+2..new_index];
                let node = Node::new(metadata, Vec::new());
                part1 += node.meta_sum;
                item.push_child(node, new_index);
    
            }
            else {
                // case #2; this child has children, so we don't know the
                // size of it yet. create a new work Item and push to stack
                let child_item = Item::new(
                    item.index + 2,
                    nchild,
                    nmeta,
                );
                stack.push(child_item);
            }
        }
    }

    // if the input is well-formatted (e.g. the numbers are coherent), this
    // will be impossible
    Err("Could not find a root node".into())
}


#[derive(Debug)]
struct Item {
    children: Vec<Node>,
    index: usize,
    nchild: usize,
    nmeta: usize
}

impl Item {
    fn new(index: usize, nchild: usize, nmeta: usize) -> Self {
        Self {
            children: Vec::with_capacity(nchild),
            index,
            nchild,
            nmeta,
        }
    }

    fn push_child(&mut self, node: Node, new_index: usize) {
        self.children.push(node);
        self.nchild -=1;
        self.index = new_index;
    }
}

#[derive(Debug)]
struct Node {
    // these 2 fields aren't needed, but in a theoretical real-world
    // application, we might be using these later for some computation
    #[allow(dead_code)]
    metadata: Vec<usize>,
    #[allow(dead_code)]
    children: Vec<Node>,

    meta_sum: usize,
}

impl Node {
    fn new(metadata: &[usize], children: Vec<Node>) -> Node {
        // calculate the 'meta sum', e.g. part2 of the challenge.
        // Node::new() is only called once all children are processed,
        // to the `meta_sum` on children is already calculated at this
        // point
        let meta_sum = if children.len() > 0 {
            metadata
                .iter()
                .flat_map(|x| children.get(*x - 1))
                .map(|x| x.meta_sum)
                .sum()
        }
        else {
            metadata.iter().sum()
        };

        Self {
            metadata: Vec::from(metadata),
            children,
            meta_sum,
        }
    }
}
