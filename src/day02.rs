use crate::{
    intcode::{compute_intcode, IntcodeMemory, Word},
    parse, CommaSep, Exercise,
};
use std::path::Path;

pub struct Day02;

impl Exercise for Day02 {
    fn part1(&self, path: &Path) {
        let mut memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();

        // comment out these initialization lines for test input
        memory[1] = 12.into();
        memory[2] = 2.into();

        memory = compute_intcode(memory);
        println!("value at ip 0: {}", memory[0]);
    }

    fn part2(&self, path: &Path) {
        let initial_memory: IntcodeMemory =
            parse::<CommaSep<Word>>(path).unwrap().flatten().collect();

        const TARGET: Word = 19_690_720;

        'stop: for noun in 0..=99 {
            for verb in 0..=99 {
                let mut memory = initial_memory.clone();
                memory[1] = noun.into();
                memory[2] = verb.into();
                memory = compute_intcode(memory);
                if memory[0] == TARGET {
                    println!("100 * noun + verb: {}", (100 * noun) + verb);
                    break 'stop;
                }
            }
        }
        println!("done part 2");
    }
}
