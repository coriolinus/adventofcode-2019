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
fn process(ip: usize, memory: &mut [Word]) -> Option<usize> {
    let (opcode, p1, p2, p3) = match memory[ip].destructure() {
        Ok((opcode, pc, pb, pa)) => (opcode, pc, pb, pa),
        Err(e) => {
            println!("{}", e);
            return None;
        }
    };
    match opcode {
        1 => {
            let out = memory[ip + 3];
            *out.value_mut(p3, memory) =
                *memory[ip + 1].value(p1, memory) + *memory[ip + 2].value(p2, memory);
            Some(4)
        }
        2 => {
            let out = memory[ip + 3];
            *out.value_mut(p3, memory) =
                *memory[ip + 1].value(p1, memory) * **memory[ip + 2].value(p2, memory);
            Some(4)
        }
        99 => None,
        _ => {
            println!("invalid opcode @ {}: {}", ip, opcode);
            None
        }
    }
}

pub fn compute_intcode(memory: &mut IntcodeMemory) {
    let mut ip = 0;
    while let Some(increment) = process(ip, memory) {
        ip += increment;
    }
}
