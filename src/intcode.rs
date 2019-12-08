use crossbeam_channel::{unbounded as channel, Receiver, Sender};
use derive_more::*;
use std::convert::TryFrom;

pub type WordInner = i32;
#[derive(
    Add,
    AddAssign,
    Clone,
    Copy,
    Debug,
    Deref,
    Display,
    Div,
    DivAssign,
    Eq,
    From,
    FromStr,
    Into,
    Mul,
    MulAssign,
    Not,
    Ord,
    PartialEq,
    PartialOrd,
    Rem,
    Sum,
)]
pub struct Word(pub WordInner);

impl From<&Word> for Word {
    fn from(w: &Word) -> Word {
        *w
    }
}

pub type IntcodeMemory = Vec<Word>;
pub type Opcode = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Position,
    Immediate,
}

impl TryFrom<Word> for Mode {
    type Error = String;

    fn try_from(value: Word) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(Mode::Position),
            1 => Ok(Mode::Immediate),
            _ => Err(format!("unrecognized mode: {}", value)),
        }
    }
}

impl Word {
    /// destructure a word into its opcode and triplet of modes
    ///
    /// output tuple is (opcode, param1, param2, param3) to line up with the position
    /// of the parameters.
    pub fn destructure(mut self: Word) -> Result<(Opcode, Mode, Mode, Mode), String> {
        let opcode = *(self % 100) as Opcode;
        self /= 100;
        let pc = Mode::try_from(self % 10)?;
        self /= 10;
        let pb = Mode::try_from(self % 10)?;
        self /= 10;
        let pa = Mode::try_from(self % 10)?;
        self /= 10;
        if *self == 0 {
            Ok((opcode, pc, pb, pa))
        } else {
            Err(format!(
                "could not destructure word as opcode: extra digits: {}",
                self
            ))
        }
    }

    fn value<'a>(&'a self, mode: Mode, memory: &'a [Word]) -> WordInner {
        use Mode::*;
        match mode {
            Position => memory[self.0 as usize].0,
            Immediate => self.0,
        }
    }

    fn value_mut<'a>(&self, mode: Mode, memory: &'a mut [Word]) -> &'a mut WordInner {
        use Mode::*;
        match mode {
            Position => &mut memory[self.0 as usize].0,
            Immediate => panic!("attempt to mutate an immediate value"),
        }
    }
}

#[derive(Debug, Default)]
pub struct Intcode {
    ip: usize,
    memory: IntcodeMemory,
    halted: bool,
    inputs: Option<Receiver<WordInner>>,
    outputs: Option<Sender<WordInner>>,
    output_ips: Option<Sender<usize>>,
    halts: Option<Sender<usize>>,
}

impl Intcode {
    pub fn new<Iter>(memory: Iter) -> Intcode
    where
        Iter: IntoIterator,
        Word: From<Iter::Item>,
    {
        Intcode {
            memory: memory.into_iter().map(Word::from).collect(),
            ..Intcode::default()
        }
    }

    pub fn with_inputs(mut self, inputs: Receiver<WordInner>) -> Self {
        self.inputs = Some(inputs);
        self
    }

    pub fn with_outputs(mut self, outputs: Sender<WordInner>) -> Self {
        self.outputs = Some(outputs);
        self
    }

    /// sends the instruction pointer location for any output
    pub fn with_output_ips(mut self, output_ips: Sender<usize>) -> Self {
        self.output_ips = Some(output_ips);
        self
    }

    /// sends the instruction pointer location for any halt
    pub fn with_halts(mut self, halts: Sender<usize>) -> Self {
        self.halts = Some(halts);
        self
    }

    // convenience fn to initialize with static inputs
    pub fn using_inputs(self, inputs: &[WordInner]) -> Self {
        let (sender, receiver) = channel();
        for input in inputs {
            sender.send(*input).unwrap();
        }
        self.with_inputs(receiver)
    }

    fn apply3<F>(&mut self, p1: Mode, p2: Mode, p3: Mode, operation: F)
    where
        F: FnOnce(WordInner, WordInner) -> WordInner,
    {
        let p1v = self.memory[self.ip + 1].value(p1, &self.memory);
        let p2v = self.memory[self.ip + 2].value(p2, &self.memory);
        let out = self.memory[self.ip + 3];
        *out.value_mut(p3, &mut self.memory) = operation(p1v, p2v);
        self.ip += 4;
    }

    fn jumpif<F>(&mut self, p1: Mode, p2: Mode, condition: F)
    where
        F: FnOnce(WordInner) -> bool,
    {
        // jump if condition is true
        let test = self.memory[self.ip + 1].value(p1, &self.memory);
        let ipval = self.memory[self.ip + 2].value(p2, &self.memory);
        #[cfg(feature = "intcode-debug")]
        dbg!("jump-if", self.ip, test, ipval);
        if condition(test) {
            self.ip = ipval as usize;
        } else {
            self.ip += 3;
        }
    }

    fn tick(&mut self) -> Result<bool, String> {
        if self.ip >= self.memory.len() {
            #[cfg(feature = "intcode-debug")]
            println!("ip overran memory at {}", self.ip);
            if let Some(halts) = &self.halts {
                if let Err(err) = halts.send(self.ip) {
                    if cfg!(feature = "intcode-debug") {
                        println!("err sending halt: {}", err);
                    }
                };
            }
            return Err(format!("ip overran memory at {}", self.ip));
        }
        if self.halted {
            return Ok(false);
        }
        let (opcode, p1, p2, p3) = self.memory[self.ip].destructure()?;
        match opcode {
            1 => {
                // add
                self.apply3(p1, p2, p3, |a, b| a + b);
            }
            2 => {
                // mul
                self.apply3(p1, p2, p3, |a, b| a * b);
            }
            3 => {
                // input
                if let Some(inputs) = &self.inputs {
                    let input = inputs
                        .recv_timeout(std::time::Duration::new(1, 0))
                        .map_err(|err| {
                            self.halted = true;
                            format!("abort: needed input at ip {} but errored: {}", self.ip, err)
                        })?;
                    #[cfg(feature = "intcode-debug")]
                    println!("input at ip {}: {}", self.ip, input);
                    let out = self.memory[self.ip + 1];
                    *out.value_mut(p1, &mut self.memory) = input.into();
                    self.ip += 2;
                } else {
                    return Err(format!("input at {} but no input stream set", self.ip));
                }
            }
            4 => {
                // output
                let val = self.memory[self.ip + 1].value(p1, &self.memory);
                #[cfg(feature = "intcode-debug")]
                println!("output at ip {}: {}", self.ip, val);
                if let Some(outputs) = &self.outputs {
                    outputs.send(val).unwrap();
                } else {
                    self.halted = true;
                    return Err(format!(
                        "output at {} ({}) but no output stream set",
                        self.ip, val
                    ));
                }
                if let Some(oips) = &self.output_ips {
                    if let Err(err) = oips.send(self.ip) {
                        if cfg!(feature = "intcode-debug") {
                            println!("err sending oip: {}", err);
                        }
                    }
                }
                self.ip += 2;
            }
            5 => {
                // jump if true
                self.jumpif(p1, p2, |test| test != 0);
            }
            6 => {
                // jump if false
                self.jumpif(p1, p2, |test| test == 0);
            }
            7 => {
                // less than
                self.apply3(p1, p2, p3, |a, b| if a < b { 1 } else { 0 });
            }
            8 => {
                // equals
                self.apply3(p1, p2, p3, |a, b| if a == b { 1 } else { 0 });
            }
            99 => {
                #[cfg(feature = "intcode-debug")]
                println!("program halt at ip {}", self.ip);
                self.halted = true;
                if let Some(halts) = &self.halts {
                    if let Err(err) = halts.send(self.ip) {
                        if cfg!(feature = "intcode-debug") {
                            println!("err sending halt: {}", err);
                        }
                    }
                }
            }
            _ => {
                return Err(format!("invalid opcode @ {}: {}", self.ip, opcode));
            }
        }
        Ok(true)
    }

    // run this computer until program completion
    pub fn run(&mut self) -> Result<(), String> {
        while self.tick()? {}
        #[cfg(feature = "intcode-debug")]
        println!("intcode run complete");
        Ok(())
    }

    // run this computer into program completion,
    // collecting the outputs into a vector
    pub fn run_collect(&mut self) -> Result<Vec<WordInner>, String> {
        let (sender, receiver) = channel();
        self.outputs = Some(sender);
        self.run()?;
        // now we have to drop the sender so that we can collect the results
        // of the receiver. For this to work, we have to replace it with a
        // None value, then manually drop it.
        let sender = std::mem::replace(&mut self.outputs, None);
        std::mem::drop(sender);
        #[cfg(feature = "intcode-debug")]
        println!("dropped sender in run_collect");
        Ok(receiver.into_iter().collect())
    }
}

pub fn compute_intcode(memory: &IntcodeMemory) {
    Intcode::new(memory.iter()).run().unwrap();
}

pub fn compute_intcode_ioch(
    memory: &IntcodeMemory,
    inputs: Receiver<WordInner>,
    outputs: Sender<WordInner>,
    halts: Sender<usize>,
) -> Result<(), String> {
    let mut computer = Intcode::new(memory.iter())
        .with_inputs(inputs)
        .with_outputs(outputs)
        .with_halts(halts);

    computer.run()
}
