use crate::{geometry::Point, Exercise};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct Day;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let asteroids = parse_asteroids(path).unwrap();
        let max_x = asteroids.iter().map(|a| a.x).max().unwrap();
        let max_y = asteroids.iter().map(|a| a.y).max().unwrap();
        let mut visibility = HashMap::new();

        for asteroid in asteroids.iter() {
            let mut visible = asteroids.clone();
            for target in asteroids.iter() {
                if target == asteroid {
                    visible.remove(target);
                    continue;
                }

                let vector = *target - *asteroid;
                debug_assert!(vector.x != 0 || vector.y != 0);
                let gcd = num_integer::gcd(vector.x, vector.y);
                let step_vector = vector / gcd;
                let initial_mul = 1 + gcd;

                for mul in initial_mul.. {
                    let hidden = (step_vector * mul) + *asteroid;
                    if hidden.x < 0 || hidden.y < 0 || hidden.x > max_x || hidden.y > max_y {
                        break;
                    }
                    visible.remove(&hidden);
                }
            }
            visibility.insert(asteroid, visible.len());
        }

        let (visible, best) = visibility
            .iter()
            .map(|(asteroid, visible)| (visible, asteroid))
            .max()
            .unwrap();
        println!("best asteroid: {:?} sees {}", best, visible);
    }

    fn part2(&self, _path: &Path) {
        unimplemented!()
    }
}

pub fn parse_asteroids(path: &Path) -> std::io::Result<HashSet<Point>> {
    let reader = BufReader::new(std::fs::File::open(path)?);
    let mut out = HashSet::new();
    for (y, line) in reader.lines().enumerate() {
        let line = line?;
        out.extend(line.as_bytes().iter().enumerate().filter_map(|(x, b)| {
            if *b == b'#' {
                Some(Point {
                    x: x as i32,
                    y: y as i32,
                })
            } else {
                None
            }
        }))
    }
    Ok(out)
}
