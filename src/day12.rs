use crate::{geometry::Vector3, parse, Exercise};
use num_integer::Integer;
use std::fmt;
use std::path::Path;

pub struct Day;

#[cfg(feature = "debug")]
const SIM_DURATION: usize = 10;
#[cfg(not(feature = "debug"))]
const SIM_DURATION: usize = 1000;

macro_rules! dimension {
    ($d:ident in $moons:expr) => {
        $moons
            .iter()
            .map(|moon| moon.position.$d)
            .collect::<Vec<_>>()
    };
}

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

    fn part2(&self, path: &Path) {
        let mut moons: Vec<_> = parse::<Vector3>(path).unwrap().map(Moon::new).collect();

        let initial_x = dimension!(x in moons);
        let initial_y = dimension!(y in moons);
        let initial_z = dimension!(z in moons);

        let mut x_cycle = None;
        let mut y_cycle = None;
        let mut z_cycle = None;

        for step in 2_u64.. {
            calc_step(&mut moons);

            if x_cycle.is_none() && initial_x == dimension!(x in moons) {
                x_cycle = Some(step);
            }

            if y_cycle.is_none() && initial_y == dimension!(y in moons) {
                y_cycle = Some(step);
            }

            if z_cycle.is_none() && initial_z == dimension!(z in moons) {
                z_cycle = Some(step);
            }

            if x_cycle.is_some() && y_cycle.is_some() && z_cycle.is_some() {
                break;
            }
        }

        let x_cycle = x_cycle.unwrap();
        let y_cycle = y_cycle.unwrap();
        let z_cycle = z_cycle.unwrap();

        #[cfg(feature = "debug")]
        {
            eprintln!("xc: {}", x_cycle);
            eprintln!("yc: {}", y_cycle);
            eprintln!("zc: {}", z_cycle);
        }

        let inter = (x_cycle * y_cycle) / x_cycle.gcd(&y_cycle);
        let cycle = (inter * z_cycle) / inter.gcd(&z_cycle);

        println!("cycle length: {}", cycle);
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
