use derive_more::*;
use std::convert::TryFrom;

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
pub struct Word(pub i32);
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

#[derive(Debug)]
pub enum Output {
    Halt { ip: usize },
    Output { ip: usize, val: i32 },
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

    pub fn value<'a>(&'a self, mode: Mode, memory: &'a [Word]) -> &'a Word {
        use Mode::*;
        match mode {
            Position => &memory[self.0 as usize],
            Immediate => &self,
        }
    }

    pub fn value_mut<'a>(&self, mode: Mode, memory: &'a mut [Word]) -> &'a mut Word {
        use Mode::*;
        match mode {
            Position => &mut memory[self.0 as usize],
            Immediate => panic!("attempt to mutate an immediate value"),
        }
    }
}

// return None for a halt
fn process(
    ip: usize,
    memory: &mut [Word],
    inputs: &mut Vec<i32>,
    outputs: &mut Vec<Output>,
) -> Option<i32> {
    let (opcode, p1, p2, p3) = match memory[ip].destructure() {
        Ok((opcode, pc, pb, pa)) => (opcode, pc, pb, pa),
        Err(e) => {
            println!("{}", e);
            return None;
        }
    };
    match opcode {
        1 => {
            // add
            let out = memory[ip + 3];
            *out.value_mut(p3, memory) =
                *memory[ip + 1].value(p1, memory) + *memory[ip + 2].value(p2, memory);
            Some(4)
        }
        2 => {
            // mul
            let out = memory[ip + 3];
            *out.value_mut(p3, memory) =
                *memory[ip + 1].value(p1, memory) * **memory[ip + 2].value(p2, memory);
            Some(4)
        }
        3 => {
            // input
            match inputs.pop() {
                Some(input) => {
                    #[cfg(feature = "intcode-debug")]
                    println!("input at ip {}: {}", ip, input);
                    let out = memory[ip + 1];
                    *out.value_mut(p1, memory) = input.into();
                    Some(2)
                }
                None => {
                    println!("abort: needed input at ip {} but none were available", ip);
                    None
                }
            }
        }
        4 => {
            // output
            let val = **memory[ip + 1].value(p1, memory);
            #[cfg(feature = "intcode-debug")]
            println!("output at ip {}: {}", ip, val);
            outputs.push(Output::Output { ip, val });
            Some(2)
        }
        5 => {
            // jump if true
            let test = **memory[ip + 1].value(p1, memory);
            let ipval = **memory[ip + 2].value(p2, memory);
            let ipdiff = ipval - ip as i32;
            #[cfg(feature = "intcode-debug")]
            dbg!("jump-if-true", ip, test, ipval, ipdiff);
            if test != 0 {
                Some(ipdiff)
            } else {
                Some(3)
            }
        }
        6 => {
            // jump if false
            let test = **memory[ip + 1].value(p1, memory);
            let ipval = **memory[ip + 2].value(p2, memory);
            let ipdiff = ipval - ip as i32;
            #[cfg(feature = "intcode-debug")]
            dbg!("jump-if-false", ip, test, ipval, ipdiff);
            if test == 0 {
                Some(ipdiff)
            } else {
                Some(3)
            }
        }
        7 => {
            // less than
            let out = memory[ip + 3];
            *out.value_mut(p3, memory) =
                if **memory[ip + 1].value(p1, memory) < **memory[ip + 2].value(p2, memory) {
                    1
                } else {
                    0
                }
                .into();
            Some(4)
        }
        8 => {
            // equals
            let out = memory[ip + 3];
            *out.value_mut(p3, memory) =
                if **memory[ip + 1].value(p1, memory) == **memory[ip + 2].value(p2, memory) {
                    1
                } else {
                    0
                }
                .into();
            Some(4)
        }
        99 => {
            #[cfg(feature = "intcode-debug")]
            println!("explicit program halt at ip {}", ip);
            outputs.push(Output::Halt { ip });
            None
        }
        _ => {
            println!("invalid opcode @ {}: {}", ip, opcode);
            None
        }
    }
}

pub fn compute_intcode(memory: &mut IntcodeMemory) {
    let zs: Vec<Word> = Vec::new();
    compute_intcode_io(memory, zs);
}

pub fn compute_intcode_io<Iter, T>(memory: &mut IntcodeMemory, inputs: Iter) -> Vec<Output>
where
    Iter: IntoIterator<Item = T>,
    T: Into<i32>,
{
    let mut ip = 0;
    let mut inputs: Vec<i32> = inputs.into_iter().map(|i| i.into()).collect();
    // we reverse the inputs so we can efficiently pop
    inputs.reverse();
    let mut outputs = Vec::new();
    while let Some(increment) = process(ip, memory, &mut inputs, &mut outputs) {
        ip = (ip as i32 + increment) as usize;
    }
    if !inputs.is_empty() {
        inputs.reverse();
        println!("warn: unused inputs: {:?}", inputs);
    }
    outputs
}
