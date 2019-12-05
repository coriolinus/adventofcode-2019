use crate::{
    intcode::{compute_intcode_io, IntcodeMemory, Output, Word},
    parse, CommaSep, Exercise,
};
use std::path::Path;

pub struct Day;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let mut memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        let outputs = compute_intcode_io(&mut memory, [1].iter().cloned());
        get_diagnostic(&outputs);
    }

    fn part2(&self, path: &Path) {
        let mut memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        let outputs = compute_intcode_io(&mut memory, [5].iter().cloned());
        get_diagnostic(&outputs);
    }
}

fn get_diagnostic(outputs: &[Output]) {
    let mut diagnostic = None;
    for o in outputs {
        match o {
            Output::Output { val, .. } if *val == 0 => continue,
            Output::Output { .. } if diagnostic.is_none() => diagnostic = Some(o),
            Output::Output { ip, val } => {
                println!("warn: non-0 output after diagnostic already set");
                println!(" ip: {}; val: {}", ip, val);
            }
            Output::Halt { ip } => {
                let hip = ip;
                match diagnostic {
                    None => {
                        println!("halt without output");
                        break;
                    }
                    Some(Output::Output { ip, val }) => {
                        let oip = ip;
                        if *hip == oip + 2 {
                            println!("diagnostic code: {}", val);
                            break;
                        } else {
                            println!("diagnostic did not immediately precede halt!");
                            println!(" oip: {}; hip: {}; val: {}", oip, hip, val);
                        }
                    }
                    _ => panic!("malformed diagnostic"),
                }
            }
        }
    }
}
