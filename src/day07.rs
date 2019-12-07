use crate::{
    intcode::{compute_intcode_io, IntcodeMemory, Output, Word},
    parse, CommaSep, Exercise,
};
use std::path::Path;

pub struct Day;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        match find_optimal_phases(&memory) {
            None => println!("no optimal phase found?!"),
            Some((phases, signal)) => {
                println!("signal {} found from phases {:?}", signal, phases);
            }
        }
    }

    fn part2(&self, path: &Path) {
        let _memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
    }
}

fn compute_amplifier_stack(memory: &IntcodeMemory, phases: &[i32]) -> i32 {
    let mut signal = 0;
    for (idx, phase) in phases.iter().enumerate() {
        let output = compute_intcode_io(&mut memory.clone(), [*phase, signal].iter().cloned());
        if output.is_empty() {
            panic!(
                "no output for idx={} phase={}, phases={:?}",
                idx, phase, phases
            );
        }

        {
            use self::Output::*;
            signal = match &output[0] {
                &Halt { .. } => panic!(
                    "unexpected halt for idx={} phase={} phases={:?}",
                    idx, phase, phases
                ),
                &Output { val, .. } => val,
            };
        }
    }
    signal
}

fn find_optimal_phases(memory: &IntcodeMemory) -> Option<(Vec<i32>, i32)> {
    let mut max_signal = None;
    let mut max_phases = None;

    let mut phases: Vec<i32> = (0..=4).collect();

    permutohedron::heap_recursive(&mut phases, |phases| {
        let signal = compute_amplifier_stack(memory, phases);
        match max_signal {
            None => {
                max_signal = Some(signal);
                max_phases = Some(phases.to_owned());
            }
            Some(ms) => {
                if signal > ms {
                    max_signal = Some(signal);
                    max_phases = Some(phases.to_owned());
                }
            }
        }
    });

    max_signal.map(|ms| (max_phases.unwrap(), ms))
}
