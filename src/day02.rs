use crate::{parse, CommaSep, Exercise};
use std::path::Path;

pub struct Day02;

pub type Word = u32;
pub type IntcodeMemory = Vec<Word>;

// return true if we should continue, or false if not
fn process(ip: usize, memory: &mut [Word]) -> bool {
    match &memory[ip..ip + 4] {
        [opcode, a, b, out] => {
            // TODO
            match *opcode {
                1 => {
                    memory[*out as usize] = memory[*a as usize] + memory[*b as usize];
                    true
                }
                2 => {
                    memory[*out as usize] = memory[*a as usize] * memory[*b as usize];
                    true
                }
                99 => false,
                _ => {
                    eprintln!("invalid opcode @ {}: {}", ip, opcode);
                    false
                }
            }
        }
        [opcode] if *opcode == 99 => false,
        other => {
            eprintln!("malformed memory opcodes @ {}: {:?}", ip, other);
            false
        }
    }
}

pub fn compute_intcode(memory: &mut IntcodeMemory) {
    let mut ip = 0;
    while process(ip, memory) {
        ip += 4;
    }
}

impl Exercise for Day02 {
    fn part1(&self, path: &Path) {
        let mut memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();

        // comment out these initialization lines for test input
        memory[1] = 12;
        memory[2] = 2;

        compute_intcode(&mut memory);
        println!("value at ip 0: {}", memory[0]);
    }

    fn part2(&self, path: &Path) {
        let initial_memory: IntcodeMemory =
            parse::<CommaSep<Word>>(path).unwrap().flatten().collect();

        const TARGET: Word = 19690720;

        'stop: for noun in 0..=99 {
            for verb in 0..=99 {
                let mut memory = initial_memory.clone();
                memory[1] = noun;
                memory[2] = verb;
                compute_intcode(&mut memory);
                if memory[0] == TARGET {
                    println!("100 * noun + verb: {}", (100 * noun) + verb);
                    break 'stop;
                }
            }
        }
        println!("done part 2");
    }
}
