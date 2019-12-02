use crate::{parse, Exercise};
use std::path::Path;

pub struct Day01;

impl Exercise for Day01 {
    fn part1(&self, path: &Path) {
        let fuel: u32 = parse::<u32>(path).unwrap().map(|mass| (mass / 3) - 2).sum();
        println!("total fuel: {}", fuel);
    }

    fn part2(&self, path: &Path) {
        let fuel: u32 = parse::<u32>(path)
            .unwrap()
            .map(|module| {
                let mut fuel_sum = 0;
                let mut fuel = (module / 3) - 2;
                while fuel > 0 {
                    fuel_sum += fuel;
                    fuel /= 3;
                    fuel -= std::cmp::min(fuel, 2);
                }
                fuel_sum
            })
            .sum();
        println!("total rocket equation fuel: {}", fuel);
    }
}
