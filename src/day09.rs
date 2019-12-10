use crate::{
    intcode::{Intcode, IntcodeMemory, Word},
    parse, CommaSep, Exercise,
};
use std::path::Path;

pub struct Day;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        let mut computer = Intcode::new(memory).using_inputs(&[1]);
        match computer.run_collect() {
            Ok(outputs) => match outputs.len() {
                0 => println!("no output"),
                1 => println!("BOOST keycode: {}", outputs[0]),
                _ => {
                    println!(
                        "self-diagnostic problems: {:?}",
                        &outputs[..outputs.len() - 1]
                    );
                    println!("BOOST keycode??: {}", outputs[outputs.len() - 1]);
                }
            },
            Err(err) => println!("intcode error: {}", err),
        }
    }

    fn part2(&self, path: &Path) {
        let memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        let mut computer = Intcode::new(memory).using_inputs(&[2]);
        let coords = computer
            .run_collect()
            .expect("computation should complete successfully")[0];
        println!("coordinates: {}", coords);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quine() {
        let quine = vec![
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
        ];
        let mut computer = Intcode::new(quine.clone());
        let output = computer.run_collect().unwrap();
        assert_eq!(output, quine);
    }

    #[test]
    fn test_16_digits() {
        let mut computer = Intcode::new(vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0]);
        let output = computer.run_collect().unwrap();
        assert!(!output.is_empty());
        assert_eq!(output[0].to_string().len(), 16);
    }

    #[test]
    fn test_big_number() {
        let program = vec![104, 1125899906842624, 99];
        let expect = program[1];
        let mut computer = Intcode::new(program);
        let output = computer.run_collect().unwrap();
        assert_eq!(output[0], expect);
    }
}
