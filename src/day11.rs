use crate::{
    geometry::{Direction, Point},
    intcode::{Intcode, IntcodeMemory, Word},
    parse, CommaSep, Exercise,
};
use crossbeam_channel::{unbounded as channel, Receiver, Sender};
use std::collections::HashSet;
use std::path::Path;
use std::thread;

pub struct Day;

const HULL_SIZE: usize = 1024;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        let (camera, receiver) = channel();
        let (sender, controls) = channel();
        let mut computer = Intcode::new(memory)
            .with_inputs(receiver)
            .with_outputs(sender);
        // we may as well send that computer off to do its own thing now;
        // it'll be ready and waiting to process instructions as soon as
        // we're ready to tell it its state
        thread::spawn(move || computer.run());

        let mut hull = vec![vec![false; HULL_SIZE]; HULL_SIZE];
        let mut robot = Robot::new(HULL_SIZE / 2, HULL_SIZE / 2, camera, controls);

        let painted = robot.run(&mut hull).unwrap();
        println!("painted {} hull panels", painted.len());
    }

    fn part2(&self, path: &Path) {
        let memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        let (camera, receiver) = channel();
        let (sender, controls) = channel();
        let mut computer = Intcode::new(memory)
            .with_inputs(receiver)
            .with_outputs(sender);
        thread::spawn(move || computer.run());

        let mut hull = vec![vec![false; HULL_SIZE]; HULL_SIZE];
        hull[HULL_SIZE / 2][HULL_SIZE / 2] = true;
        let mut robot = Robot::new(HULL_SIZE / 2, HULL_SIZE / 2, camera, controls);

        robot.run(&mut hull).unwrap();

        let mut min_x = None;
        let mut max_x = None;
        let mut min_y = None;
        let mut max_y = None;
        for (y, row) in hull.iter().enumerate() {
            for (x, val) in row.iter().enumerate() {
                if *val {
                    if min_x.is_none() || x < min_x.unwrap() {
                        min_x = Some(x);
                    }
                    if max_x.is_none() || x > max_x.unwrap() {
                        max_x = Some(x);
                    }
                    if min_y.is_none() || y < min_y.unwrap() {
                        min_y = Some(y);
                    }
                    if max_y.is_none() || y > max_y.unwrap() {
                        max_y = Some(y);
                    }
                }
            }
        }

        for row in hull[min_y.expect("min_y")..=max_y.expect("max_y")].iter().rev() {
            for val in &row[min_x.expect("min_x")..=max_x.expect("max_x")] {
                if *val {
                    print!("#");
                } else {
                    print!(" ");
                }
            }
            println!();
        }
    }
}

pub struct Robot {
    location: Point,
    facing: Direction,
    camera: Sender<Word>,
    controls: Receiver<Word>,
}

impl Robot {
    pub fn new(x: usize, y: usize, camera: Sender<Word>, controls: Receiver<Word>) -> Robot {
        Robot {
            location: Point::new(x as i32, y as i32),
            facing: Direction::Up,
            camera,
            controls,
        }
    }

    pub fn run(&mut self, hull: &mut Vec<Vec<bool>>) -> Result<HashSet<Point>, String> {
        let mut painted = HashSet::new();

        let mut loc = self.loc()?;
        let mut existing_color = hull[loc.1][loc.0];
        while let Ok(_) = self.camera.send(if existing_color { 1 } else { 0 }) {
            #[cfg(feature = "debug")]
            dbg!(self.location, existing_color);

            if let Ok(color_inst) = self.controls.recv() {
                let new_color = match color_inst {
                    0 => false,
                    1 => true,
                    color => return Err(format!("unexpected paint color: {}", color)),
                };

                #[cfg(feature = "debug")]
                dbg!(new_color);

                painted.insert(self.location);

                hull[loc.1][loc.0] = new_color;
            } else {
                break;
            }

            self.facing = match self.controls.recv().expect("if got color must get turn") {
                0 => self.facing.turn_left(),
                1 => self.facing.turn_right(),
                turn => return Err(format!("unexpected turn instruction: {}", turn)),
            };
            self.location = self.location + self.facing.deltas();

            #[cfg(feature = "debug")]
            dbg!(self.facing);

            loc = self.loc()?;
            existing_color = hull[loc.1][loc.0];
        }
        Ok(painted)
    }

    /// (x, y) as long as those coordinates are in bounds
    pub fn loc(&self) -> Result<(usize, usize), String> {
        if self.location.x < 0
            || self.location.y < 0
            || self.location.x >= HULL_SIZE as i32
            || self.location.y >= HULL_SIZE as i32
        {
            Err(format!("out of bounds: {:?}", self.location))
        } else {
            Ok((self.location.x as usize, self.location.y as usize))
        }
    }
}
