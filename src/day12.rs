use crate::{geometry::Vector3, parse, Exercise};

use std::fmt;
use std::path::Path;

pub struct Day;

#[cfg(feature="debug")]
const SIM_DURATION: usize = 10;
#[cfg(not(feature="debug"))]
const SIM_DURATION: usize = 1000;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let mut moons: Vec<_> = parse::<Vector3>(path).unwrap().map(Moon::new).collect();
        for step in 0..SIM_DURATION {
            if cfg!(feature = "debug") && step <= 10 {
                if step <= 10 {
                    println!("After {} steps:", step);
                    for moon in &moons {
                        println!("{}", moon);
                    }
                    println!();
                }
            }
            calc_step(&mut moons);
        }
        println!(
            "sum total energy after {} steps: {}",
            SIM_DURATION,
            moons.iter().map(Moon::total_energy).sum::<i32>()
        );
    }

    fn part2(&self, _path: &Path) {
        unimplemented!()
    }
}

struct Moon {
    position: Vector3,
    velocity: Vector3,
}

impl Moon {
    fn new(position: Vector3) -> Moon {
        Moon {
            position,
            velocity: Vector3::default(),
        }
    }

    fn potential_energy(&self) -> i32 {
        self.position.abs_sum()
    }

    fn kinetic_energy(&self) -> i32 {
        self.velocity.abs_sum()
    }

    fn total_energy(&self) -> i32 {
        self.potential_energy() * self.kinetic_energy()
    }
}

impl fmt::Display for Moon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<pos={}, vel={}>", self.position, self.velocity)
    }
}

fn ordval(ord: std::cmp::Ordering) -> i32 {
    use std::cmp::Ordering::*;
    match ord {
        Less => -1,
        Equal => 0,
        Greater => 1,
    }
}

fn calc_step(moons: &mut [Moon]) {
    // update velocities by applying gravity
    for i in 0..moons.len() {
        for j in 0..moons.len() {
            moons[i].velocity.x += ordval(moons[j].position.x.cmp(&moons[i].position.x));
            moons[i].velocity.y += ordval(moons[j].position.y.cmp(&moons[i].position.y));
            moons[i].velocity.z += ordval(moons[j].position.z.cmp(&moons[i].position.z));
        }
    }

    // update positions by applying velocity
    for moon in moons.iter_mut() {
        moon.position += moon.velocity;
    }
}
