use crate::{parse, Exercise};
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

pub struct Day;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let system = System::new(parse::<OrbitRelation>(path).unwrap())
            .expect("failed to organize the solar system");
        println!("com name: {}", system.bodies[system.com.unwrap()].name);
        println!("sum of orbits: {}", system.sum_orbits());
    }

    fn part2(&self, _path: &Path) {
        unimplemented!()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct OrbitRelation {
    com: String,
    orbiter: String,
}

impl FromStr for OrbitRelation {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bodies: Vec<_> = s.split(')').collect();
        if bodies.len() == 2 {
            Ok(OrbitRelation {
                com: bodies[0].to_string(),
                orbiter: bodies[1].to_string(),
            })
        } else {
            Err("wrong number of bodies in relation")
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Body {
    name: String,
    children: Vec<usize>,
    parent: Option<usize>,
}

#[derive(Default, Debug)]
pub struct System {
    bodies: Vec<Body>,
    name_map: HashMap<String, usize>,
    com: Option<usize>,
}

impl System {
    pub fn new(relations: impl IntoIterator<Item = OrbitRelation>) -> Result<System, String> {
        // create a map of names
        let mut names: HashMap<String, Vec<String>> = HashMap::new();
        for OrbitRelation { com, orbiter } in relations.into_iter() {
            names.entry(orbiter.clone()).or_default();
            names.entry(com).or_default().push(orbiter);
        }

        // initialize the system
        let mut sys = System {
            bodies: vec![Body::default(); names.len()],
            name_map: HashMap::with_capacity(names.len()),
            ..System::default()
        };

        for (idx, (name, _)) in names.iter().enumerate() {
            sys.name_map.insert(name.clone(), idx);
            sys.bodies[idx].name = name.clone();
        }

        // we now have a composite map from names to bodies, whose indices are Copy
        // let's insert the familial relationships
        for (name, children) in names.iter() {
            let com_idx = sys.name_map[name];
            for child in children {
                let child_idx = *sys
                    .name_map
                    .get(child)
                    .ok_or_else(|| format!("child {} not in system name map", child))?;
                // add parent relationships
                sys.bodies[child_idx].parent = Some(com_idx);
                // add child relationships
                sys.bodies[com_idx].children.push(child_idx);
            }
        }

        // sanity check: no more than one body is the system center of mass
        for (idx, body) in sys.bodies.iter().enumerate() {
            if body.parent.is_none() {
                match sys.com {
                    None => sys.com = Some(idx),
                    Some(com_idx) => {
                        return Err(format!(
                            "more than one unparented body in system: {}, {}",
                            sys.bodies[idx].name, sys.bodies[com_idx].name
                        ))
                    }
                }
            }
        }
        // sanity check: there is a system center of mass
        if sys.com.is_none() {
            return Err("no overall system center of mass".into());
        }

        Ok(sys)
    }

    fn count_orbits(&self, mut idx: usize) -> usize {
        let mut count = 0;
        while let Some(parent_idx) = self.bodies[idx].parent {
            count += 1;
            idx = parent_idx;
        }
        count
    }

    pub fn sum_orbits(&self) -> usize {
        (0..self.bodies.len())
            .map(|idx| self.count_orbits(idx))
            .sum()
    }
}
