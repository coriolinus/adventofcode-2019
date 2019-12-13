use crate::{
    intcode::{channel, Intcode, IntcodeMemory, Word},
    parse, CommaSep, Exercise,
};
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::path::Path;
use std::thread;

pub struct Day;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        let (output, to_screen) = channel();
        let mut computer = Intcode::new(memory).with_outputs(output);
        thread::spawn(move || {
            computer.run().unwrap();
        });

        let mut blocks = 0;
        while let Ok(_x) = to_screen.recv() {
            let _y = to_screen.recv().unwrap();
            let tile: Tile = to_screen.recv().unwrap().try_into().unwrap();
            if tile == Tile::Block {
                blocks += 1;
            }
        }
        println!("total blocks: {}", blocks);
    }

    fn part2(&self, _path: &Path) {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Tile::*;
        write!(
            f,
            "{}",
            match self {
                Empty => " ",
                Wall => "█",
                Block => "▢",
                Paddle => "―",
                Ball => "⭘",
            }
        )
    }
}

impl TryFrom<Word> for Tile {
    type Error = String;

    fn try_from(value: Word) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Tile::Empty),
            1 => Ok(Tile::Wall),
            2 => Ok(Tile::Block),
            3 => Ok(Tile::Paddle),
            4 => Ok(Tile::Ball),
            _ => Err(format!("unrecognized tile: {}", value)),
        }
    }
}
