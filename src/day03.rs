use crate::{geometry::*, parse, CommaSep, Exercise};
use std::collections::BTreeMap;
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
        if isects.is_empty() {
            println!("no intersections");
            return;
        }
        println!(
            "md of nearest isect: {} ({:?})",
            isects[0].manhattan(),
            isects[0]
        );
    }

    fn part2(&self, path: &Path) {
        let wires: Vec<Vec<Line>> = parse::<CommaSep<Trace>>(path)
            .unwrap()
            .map(|cs| cs.0)
            .map(|t| follow(&t))
            .collect();
        if wires.len() < 2 {
            println!("too few wires");
            return;
        }
        let mut isects = intersections_steps(&wires[0], &wires[1]);
        // we don't care about intersections at the origin
        isects.remove(&0);
        match isects.keys().next() {
            None => {
                println!("no intersections");
            }
            Some(first_key) => {
                let point = isects[first_key];
                println!("steps of nearest isect: {} ({:?})", first_key, point);
            }
        }
    }
}

fn intersections_steps(ap: &[Line], bp: &[Line]) -> BTreeMap<i32, Point> {
    let mut isects = BTreeMap::new();

    let mut a_steps = 0;
    for a in ap {
        let mut b_steps = 0;

        for b in bp {
            if let Some(isect) = intersect(a, b) {
                let steps =
                    a_steps + b_steps + (isect - a.from).manhattan() + (isect - b.from).manhattan();
                isects.entry(steps).or_insert(isect);
            }
            b_steps += b.manhattan_len();
        }
        a_steps += a.manhattan_len();
    }
    isects
}
