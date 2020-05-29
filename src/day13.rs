use crate::{
    geometry::Point,
    intcode::{channel, Intcode, IntcodeMemory, Word},
    ordering_value, parse, CommaSep, Exercise,
};
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::path::Path;
use std::thread;
use term_cursor::{clear, set_pos};

#[cfg(feature = "debug")]
use std::io::Write;

pub const INFO_X: i32 = 40;
pub const SCORE_Y: i32 = 3;
pub const DEBUG_Y: i32 = 5;

pub struct Day;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        let (output, to_screen) = channel();
        let mut computer = Intcode::new(memory).with_outputs(output);
        thread::spawn(move || {
            computer.run().unwrap();
        });

        clear().unwrap();
        let mut blocks = 0;
        let mut max_y = 0;
        while let Ok(x) = to_screen.recv() {
            let y = to_screen.recv().unwrap();
            if y > max_y {
                max_y = y;
            }
            let tile: Tile = to_screen.recv().unwrap().try_into().unwrap();
            if tile == Tile::Block {
                blocks += 1;
            }
            #[cfg(not(feature = "debug"))]
            {
                set_pos(x as i32 + 1, y as i32 + 1).unwrap();
                print!("{}", tile);
            }
            #[cfg(feature = "debug")]
            println!("{:?} @ ({}, {})", tile, x, y);
        }
        #[cfg(not(feature = "debug"))]
        set_pos(0, max_y as i32 + 1).unwrap();
        println!("\ntotal blocks: {}", blocks);
    }

    fn part2(&self, path: &Path) {
        let mut memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        memory[0] = 2;
        let (outputs, to_screen) = channel();
        let (joystick, inputs) = channel();
        let mut computer = Intcode::new(memory)
            .with_outputs(outputs)
            .with_inputs(inputs);
        // run the computer
        thread::spawn(move || {
            computer.run().unwrap();
        });

        // handle output in its own thread, so we don't need to worry about
        // synchronization with the input
        #[cfg(feature = "debug")]
        clear().unwrap();
        let mut score = 0;
        let mut ball_pos;
        let mut paddle_pos = Point::default();

        while let Ok(x) = to_screen.recv() {
            let y = to_screen.recv().unwrap();
            let val = to_screen.recv().unwrap();

            if x == -1 && y == 0 {
                score = val;

                #[cfg(feature = "debug")]
                {
                    set_pos(INFO_X, SCORE_Y).unwrap();
                    print!("score: {}", val);
                }
            } else {
                let tile: Tile = val.try_into().unwrap();
                #[cfg(feature = "debug")]
                {
                    set_pos(x as i32 + 1, y as i32 + 1).unwrap();
                    print!("{}", tile);
                }
                match tile {
                    Tile::Ball => {
                        ball_pos = Point::new(x as i32, y as i32);

                        // the ball has to update once every tick, so let's send our inputs here

                        let movement = ordering_value(ball_pos.x.cmp(&paddle_pos.x));
                        if let Err(err) = joystick.send(movement.into()) {
                            set_pos(1, paddle_pos.y + 3).unwrap();
                            println!("joystick send: {}", err);
                            return;
                        };

                        #[cfg(feature = "debug")]
                        {
                            set_pos(INFO_X, DEBUG_Y).unwrap();
                            print!("ball: {:?}", ball_pos);
                            set_pos(INFO_X, DEBUG_Y + 1).unwrap();
                            print!("paddle: {:?}", paddle_pos);
                            // set_pos(INFO_X, DEBUG_Y + 2).unwrap();
                            // print!("isect: {:?}", isect);
                            set_pos(INFO_X, DEBUG_Y + 3).unwrap();
                            print!("joystick: {:2}", movement);

                            std::io::stdout().flush().unwrap();
                            thread::sleep(std::time::Duration::from_millis(700));
                        }
                    }
                    Tile::Paddle => {
                        paddle_pos = Point::new(x as i32, y as i32);
                    }
                    _ => {}
                }
            }
        }
        #[cfg(feature = "debug")]
        set_pos(1, paddle_pos.y + 3).unwrap();
        println!("score on clear: {}", score);
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
                Block => "X",
                Paddle => "―",
                Ball => "o",
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
