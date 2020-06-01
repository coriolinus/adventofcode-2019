use crate::{
    intcode::{Intcode, IntcodeMemory, Word},
    parse, CommaSep, Exercise,
};
use std::path::Path;

pub struct Day;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();

        let mut computer = Intcode::new(memory);
        let ascii_data = computer
            .run_collect()
            .unwrap()
            .iter()
            .map(|c| *c as u8 as char)
            .collect::<String>();
        println!("{}", ascii_data);

        let mut data = Vec::new();
        let mut current_row = Vec::new();
        for ch in ascii_data.chars() {
            match ch {
                '.' => current_row.push(Tile::Empty),
                '#' | 'v' | '<' | '^' | '>' => current_row.push(Tile::Scaffolding),
                '\n' => {
                    let len = current_row.len();
                    data.push(current_row);
                    current_row = Vec::with_capacity(len);
                }
                _ => unreachable!("unexpected symbol in ascii data"),
            }
        }
        if !current_row.is_empty() {
            data.push(current_row);
        }

        let mut alignment_params = 0;
        for (y, row) in data.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if *tile == Tile::Scaffolding && is_intersection(x, y, &data) {
                    alignment_params += x * y;
                }
            }
        }

        println!("sum of alignment params: {}", alignment_params);
    }

    fn part2(&self, _path: &Path) {}
}

fn is_intersection<D, R>(x: usize, y: usize, data: D) -> bool
where
    D: AsRef<[R]>,
    R: AsRef<[Tile]>,
{
    if data.as_ref()[y].as_ref()[x] != Tile::Scaffolding {
        return false;
    }
    if x > 0 && data.as_ref()[y].as_ref()[x - 1] != Tile::Scaffolding {
        return false;
    }
    if x < data.as_ref()[y].as_ref().len() - 1
        && data.as_ref()[y].as_ref()[x + 1] != Tile::Scaffolding
    {
        return false;
    }
    if y > 0 && data.as_ref()[y - 1].as_ref()[x] != Tile::Scaffolding {
        return false;
    }
    if y < data.as_ref().len() - 1 && data.as_ref()[y + 1].as_ref()[x] != Tile::Scaffolding {
        return false;
    }
    true
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Tile {
    Empty,
    Scaffolding,
}
