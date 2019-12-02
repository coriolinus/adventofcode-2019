use crate::{parse, CommaSep, Exercise};
use std::path::Path;

pub struct Day02;

type Word = u32;

// return true if we should continue, or false if not
fn process(position: usize, program: &mut [Word]) -> bool {
    match &program[position..position + 4] {
        [opcode, a, b, out] => {
            // TODO
            match *opcode {
                1 => {
                    program[*out as usize] = program[*a as usize] + program[*b as usize];
                    true
                }
                2 => {
                    program[*out as usize] = program[*a as usize] * program[*b as usize];
                    true
                }
                99 => false,
                _ => {
                    eprintln!("invalid opcode @ {}: {}", position, opcode);
                    false
                }
            }
        }
        [opcode] if *opcode == 99 => false,
        other => {
            eprintln!("malformed program opcodes @ {}: {:?}", position, other);
            false
        }
    }
}

impl Exercise for Day02 {
    fn part1(&self, path: &Path) {
        let mut program: Vec<Word> = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        let mut position = 0;
        
        // comment out these initialization lines for test input
        program[1] = 12;
        program[2] = 2;

        while process(position, &mut program) {
            position += 4;
        }
        println!("value at position 0: {}", program[0]);
    }

    fn part2(&self, _: &Path) {
        unimplemented!()
    }
}
