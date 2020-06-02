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

    fn part2(&self, path: &Path) {
        let memory = {
            let mut memory: IntcodeMemory =
                parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
            assert_eq!(memory[0], 1, "intcode begins with unexpected instruction");
            memory[0] = 2;
            memory
        };

        let inputs = PART_2_MOVEMENT_LOGIC
            .as_bytes()
            .iter()
            .map(|b| *b as Word)
            .collect::<Vec<_>>();

        let mut computer = Intcode::new(memory).using_inputs(&inputs);
        let output = computer.run_collect().unwrap();
        let (other_output, space_dust) = output.split_at(output.len() - 1);
        let space_dust = space_dust[0];
        if cfg!(featur = "debug") {
            println!(
                "{}",
                other_output
                    .iter()
                    .map(|c| *c as u8 as char)
                    .collect::<String>()
            );
        }
        println!("dust collected: {}", space_dust);
    }
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

// I arrived at this pattern by printing out the map and tracing correspondences.
// While it's certainly possible to figure this out automatically, it would be
// pretty tough; I did things the simple way this time.
const PART_2_MOVEMENT_LOGIC: &'static str = "A,C,A,B,B,A,C,B,C,C
L,8,R,10,L,8,R,8
L,8,R,6,R,6,R,10,L,8
L,12,R,8,R,8
n
";
