use crate::{
    intcode::{Intcode, IntcodeMemory, Word},
    parse, CommaSep, Exercise,
};
use std::path::Path;

pub struct Day;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        let (halt_sender, halt_receiver) = crossbeam_channel::unbounded();
        let (oip_sender, oip_receiver) = crossbeam_channel::unbounded();

        let mut computer = Intcode::new(memory)
            .using_inputs(&[1])
            .with_halts(halt_sender)
            .with_output_ips(oip_sender);

        let outputs = computer.run_collect().unwrap();
        std::mem::drop(computer); // so iters complete
        let halts = halt_receiver.into_iter().collect::<Vec<_>>();
        let oips = oip_receiver.into_iter().collect::<Vec<_>>();

        get_diagnostic(&outputs, &oips, &halts);
    }

    fn part2(&self, path: &Path) {
        let memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        let (halt_sender, halt_receiver) = crossbeam_channel::unbounded();
        let (oip_sender, oip_receiver) = crossbeam_channel::unbounded();

        let mut computer = Intcode::new(memory)
            .using_inputs(&[5])
            .with_halts(halt_sender)
            .with_output_ips(oip_sender);

        let outputs = computer.run_collect().unwrap();
        std::mem::drop(computer); // so iters complete
        let halts = halt_receiver.into_iter().collect::<Vec<_>>();
        let oips = oip_receiver.into_iter().collect::<Vec<_>>();

        get_diagnostic(&outputs, &oips, &halts);
    }
}

fn get_diagnostic(outputs: &[i32], oips: &[usize], halts: &[usize]) {
    if halts.is_empty() {
        println!("need a halt; got none");
        return;
    }
    if halts.len() > 1 {
        println!("warn: need 1 halt; got {}", halts.len());
    }
    if outputs.is_empty() {
        println!("need at least 1 output; got none");
        return;
    }
    if oips.is_empty() {
        println!("need at least 1 output ip; got none");
        return;
    }
    if outputs.len() != oips.len() {
        println!(
            "qty outputs ({}) didn't match qty oips ({})",
            outputs.len(),
            oips.len()
        );
        return;
    }

    // check diagnostics
    for (idx, (output, oip)) in outputs.iter().zip(oips).enumerate() {
        if *output == 0 && idx < outputs.len() - 1 {
            // diagnostic ok
            continue;
        }
        if idx != outputs.len() - 1 {
            println!(
                "warn: diagnostic at ip {} failed with code {}",
                *oip, *output
            );
        }
    }

    if halts[0] != oips[oips.len() - 1] + 2 {
        println!("warn: final halt not immediately preceded by final output");
    }

    println!("diagnostic code: {}", outputs[outputs.len() - 1]);
}
