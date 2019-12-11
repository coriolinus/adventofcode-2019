use crate::{geometry::Point, Exercise};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct Day;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let asteroids = parse_asteroids(path).unwrap();
        let mut visibility = HashMap::new();

        for asteroid in asteroids.iter().cloned() {
            visibility.insert(asteroid, compute_visible_from(asteroid, &asteroids).len());
        }

        let (visible, best) = visibility
            .iter()
            .map(|(asteroid, visible)| (visible, asteroid))
            .max()
            .unwrap();
        println!("best asteroid: {:?} sees {}", best, visible);
    }

    fn part2(&self, path: &Path) {
        let asteroids = parse_asteroids(path).unwrap();

        #[cfg(not(feature = "debug"))]
        const LASER: Point = Point::new(19, 11);
        #[cfg(feature = "debug")]
        const LASER: Point = Point::new(11, 13);

        #[cfg(feature = "debug")]
        dbg!(LASER);

        const BET_IDX: usize = 199;
        // strictly speaking, we should iterate: compute visibility, sort,
        // then proceed with the next visible set. We don't actually need that,
        // though: we know the index is less than the size of the initial visible set
        let mut visible: Vec<_> = compute_visible_from(LASER, &asteroids)
            .into_iter()
            .collect();

        debug_assert!(visible.len() > BET_IDX);

        // sort by clockwiseness from up
        visible.sort_by_key(|asteroid| {
            let vector = *asteroid - LASER;
            // Rust's atan2 function measures counter-clockwise from positive x
            // we need to measure clockwise from negative y, so we need to remap
            // these coordinates.
            let y = -vector.y as f64;
            let x = vector.x as f64;

            let mut v = std::f64::consts::FRAC_PI_2 - y.atan2(x);
            if v < 0.0 {
                v += 2.0 * std::f64::consts::PI;
            }

            noisy_float::types::r64(v)
        });

        #[cfg(feature = "debug")]
        {
            for (idx, asteroid) in visible.iter().enumerate().take(3) {
                println!(
                    "{}th vaporized: {:?}; output: {}",
                    idx + 1,
                    asteroid,
                    asteroid.x * 100 + asteroid.y
                );
            }
        }
        println!(
            "{}th vaporized: {:?}; output: {}",
            BET_IDX + 1,
            visible[BET_IDX],
            visible[BET_IDX].x * 100 + visible[BET_IDX].y
        );
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

fn compute_visible_from(asteroid: Point, asteroids: &HashSet<Point>) -> HashSet<Point> {
    let max_x = asteroids.iter().map(|a| a.x).max().unwrap();
    let max_y = asteroids.iter().map(|a| a.y).max().unwrap();

    let mut visible = asteroids.clone();
    for target in asteroids.iter().cloned() {
        if target == asteroid {
            visible.remove(&target);
            continue;
        }

        let vector = target - asteroid;
        debug_assert!(vector.x != 0 || vector.y != 0);
        let gcd = num_integer::gcd(vector.x, vector.y);
        let step_vector = vector / gcd;
        let initial_mul = 1 + gcd;

        for mul in initial_mul.. {
            let hidden = (step_vector * mul) + asteroid;
            if hidden.x < 0 || hidden.y < 0 || hidden.x > max_x || hidden.y > max_y {
                break;
            }
            visible.remove(&hidden);
        }
    }
    visible
}
