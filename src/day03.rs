use crate::{geometry::*, parse, CommaSep, Exercise};
use std::path::Path;

pub struct Day03;

impl Exercise for Day03 {
    fn part1(&self, path: &Path) {
        let wires: Vec<Vec<Line>> = parse::<CommaSep<Trace>>(path)
            .unwrap()
            .map(|cs| cs.0)
            .map(|t| follow(&t))
            .collect();
        if wires.len() < 2 {
            println!("too few wires");
            return;
        }
        let mut isects = intersections_naive(&wires[0], &wires[1]);
        isects.sort_by_key(Point::manhattan);
        while isects.len() > 1 && isects[0].manhattan() == 0 {
            isects.remove(0);
        }
                        if isects.len() < 1 {
                            println!("no intersections");
                            return;
                        }
        println!(
            "md of nearest isect: {} ({:?})",
            isects[0].manhattan(),
            isects[0]
        );
    }

    fn part2(&self, _: &Path) {
        unimplemented!()
    }
}
