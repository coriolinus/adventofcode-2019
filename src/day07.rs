use crate::{
    intcode::{compute_intcode_ioch, IntcodeMemory, Word},
    parse, CommaSep, Exercise,
};
use std::path::Path;
use std::thread;

pub struct Day;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        match find_optimal_phases(&memory, (0..=4).collect()) {
            None => println!("no optimal phase found?!"),
            Some((phases, signal)) => {
                println!("signal {} found from phases {:?}", signal, phases);
            }
        }
    }

    fn part2(&self, path: &Path) {
        let memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        match find_optimal_phases(&memory, (5..=9).collect()) {
            None => println!("no optimal phase found?!"),
            Some((phases, signal)) => {
                println!("signal {} found from phases {:?}", signal, phases);
            }
        }
    }
}

fn compute_amplifier_stack(memory: &IntcodeMemory, phases: &[i32]) -> i32 {
    use crossbeam_channel::{unbounded as channel, Receiver, Sender};

    // these variables carry the sender and receiver to the next amp stage
    // note that these particular values are never used except to convince
    // the compiler that they are never used before initialization
    let (mut sender, mut receiver) = channel();
    let mut prev_sender: Sender<i32>;
    let mut prev_receiver: Receiver<i32>;
    let (halts_send, halts_recv) = channel();

    // this pair handles wraparound
    let (w_sender, w_receiver) = channel();

    for (idx, phase) in phases.iter().enumerate() {
        // set up the senders/receivers
        if idx == 0 {
            prev_sender = w_sender.clone();
            prev_receiver = w_receiver.clone();
        } else {
            prev_sender = sender;
            prev_receiver = receiver;
        }

        let (s, r) = channel();
        sender = s;
        receiver = r;

        // initialize with the phase
        prev_sender.send(*phase).unwrap();

        let mc = memory.clone();
        let prc = prev_receiver.clone();
        let sc = sender.clone();
        let hsc = halts_send.clone();
        thread::spawn(move || compute_intcode_ioch(mc, prc, sc, hsc));

        // now we can properly wrap around by forwarding messages
        // (only for part 2)
        if idx == phases.len() - 1 && *phase > 4 {
            let wsc = w_sender.clone();
            let rc = receiver.clone();
            thread::spawn(move || {
                while let Ok(signal) = rc.recv() {
                    wsc.send(signal).unwrap();
                }
            });
        }
    }

    // all nodes are set up, let's give the some input
    w_sender.send(0).unwrap();

    if phases.iter().all(|phase| *phase < 5) {
        // part 1
        receiver.recv().unwrap()
    } else {
        // part 2
        // how to we detect when the nodes have stopped processing?
        // wait for all halts senders to close
        std::mem::drop(halts_send);
        while let Ok(_) = halts_recv.recv() {}
        w_receiver.recv().unwrap()
    }
}

fn find_optimal_phases(memory: &IntcodeMemory, mut phases: Vec<i32>) -> Option<(Vec<i32>, i32)> {
    let mut max_signal = None;
    let mut max_phases = None;

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
