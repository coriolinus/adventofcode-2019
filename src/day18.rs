use crate::{
    geometry::{Map as GenericMap, Point, Traversable},
    Exercise,
};
use std::collections::{HashMap, VecDeque};
use std::convert::TryFrom;
use std::path::Path;

pub struct Day;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let explorer = Explorer::from(Map::try_from(path).unwrap());
        println!(
            "steps to claim all keys: {}",
            explorer.explore_until_all_keys_claimed()
        );
    }

    fn part2(&self, _path: &Path) {
        unimplemented!()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tile {
    Wall,
    Empty,
    Door(char),
    Key(char),
    Entrance,
}

impl From<Tile> for Traversable {
    fn from(tile: Tile) -> Traversable {
        use Tile::*;

        match tile {
            Wall => Traversable::Obstructed,
            Empty => Traversable::Free,
            Door(_) => Traversable::Obstructed,
            Key(_) => Traversable::Halt,
            Entrance => Traversable::Free,
        }
    }
}

type Map = GenericMap<Tile>;

impl Default for Tile {
    fn default() -> Self {
        Tile::Empty
    }
}

impl TryFrom<char> for Tile {
    type Error = String;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '#' => Ok(Tile::Wall),
            '.' => Ok(Tile::Empty),
            'A'..='Z' => Ok(Tile::Door(c)),
            'a'..='z' => Ok(Tile::Key(c)),
            '@' => Ok(Tile::Entrance),
            _ => Err(format!("invalid tile: {}", c)),
        }
    }
}

impl From<Tile> for char {
    fn from(t: Tile) -> char {
        use Tile::*;
        match t {
            Wall => '#',
            Empty => '.',
            Door(c) => c,
            Key(c) => c,
            Entrance => '@',
        }
    }
}

#[derive(Default, Clone)]
struct Explorer {
    map: Map,
    entrance: Point,
    position: Point,
    steps: usize,
}

impl std::fmt::Debug for Explorer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Explorer {{ position: {:?}, .. }}", self.position,)
    }
}

impl From<Map> for Explorer {
    fn from(map: Map) -> Explorer {
        let mut explorer = Explorer::default();

        map.for_each_point(|tile: &Tile, point: Point| match *tile {
            Tile::Entrance => {
                explorer.entrance = point;
                explorer.position = point;
            }
            _ => {}
        });

        explorer.map = map;
        explorer
    }
}

impl Explorer {
    fn visit(&self) -> Vec<Explorer> {
        let mut reachable_keys = HashMap::new();
        self.map.reachable_from(self.position, |tile, position| {
            if let Tile::Key(key) = *tile {
                reachable_keys.insert(key, position);
            }
            false
        });

        reachable_keys
            .iter()
            .map(|(key, position)| {
                let mut successor = self.clone();
                // move the successor to the key, taking the shortest path
                successor.position = *position;
                successor.steps += self
                    .map
                    .navigate(self.position, successor.position)
                    .unwrap()
                    .len();
                // remove this key from the map
                successor.map[successor.position] = Tile::Empty;
                // unlock any doors opened by this key
                successor.map.for_each_mut(|tile| {
                    if *tile == Tile::Door(key.to_ascii_uppercase()) {
                        *tile = Tile::Empty;
                    }
                });

                successor
            })
            .collect()
    }

    fn explore_until_all_keys_claimed(&self) -> usize {
        let mut queue = VecDeque::new();
        queue.push_back(self.clone());

        while let Some(explorer) = queue.pop_front() {
            queue.extend(explorer.visit());
            if queue.is_empty() {
                return explorer.steps;
            }
        }

        self.steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_0() -> &'static str {
        "
#########
#b.A.@.a#
#########
        "
        .trim()
    }

    fn example_1() -> &'static str {
        "
########################
#f.D.E.e.C.b.A.@.a.B.c.#
######################.#
#d.....................#
########################
        "
        .trim()
    }

    fn reachable_keys_impl(input: &str, position: Point, steps: usize) {
        let explorer = Explorer::from(Map::try_from(input).unwrap());
        let successors = explorer.visit();
        assert_eq!(successors.len(), 1);
        assert_eq!(successors[0].position, position);
        successors[0].map.for_each(|tile| {
            if *tile == Tile::Key('a') {
                panic!("key a not removed");
            }
            if *tile == Tile::Door('A') {
                panic!("door A not removed");
            }
        });
        assert_eq!(successors[0].steps, steps);
    }

    #[test]
    fn reachable_keys() {
        reachable_keys_impl(example_0(), (7, 1).into(), 2);
        reachable_keys_impl(example_1(), (17, 3).into(), 2);
    }

    #[test]
    fn test_example_0() {
        let input = example_0();
        let explorer = Explorer::from(Map::try_from(input).unwrap());
        assert_eq!(explorer.explore_until_all_keys_claimed(), 8);
    }

    #[test]
    fn test_example_1() {
        let input = example_1();
        let explorer = Explorer::from(Map::try_from(input).unwrap());
        assert_eq!(explorer.explore_until_all_keys_claimed(), 86);
    }
}
