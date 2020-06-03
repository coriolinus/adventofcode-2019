use crate::{
    geometry::{Direction, Map as GenericMap, Point},
    Exercise,
};
use std::collections::{HashSet, VecDeque};
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
    Visited,
    Entrance,
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
            Visited => '_',
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
    unclaimed_keys: HashSet<char>,
}

impl std::fmt::Debug for Explorer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Explorer {{ position: {:?}, unclaimed_keys: {:?}, .. }}",
            self.position, self.unclaimed_keys
        )
    }
}

impl From<Map> for Explorer {
    fn from(mut map: Map) -> Explorer {
        let mut explorer = Explorer::default();

        map.for_each_point(|tile: &mut Tile, point: Point| match *tile {
            Tile::Entrance => {
                explorer.entrance = point;
                explorer.position = point;
            }
            Tile::Key(k) => {
                explorer.unclaimed_keys.insert(k);
            }
            _ => {}
        });

        explorer.map = map;
        explorer
    }
}

impl Explorer {
    fn visit(mut self) -> Vec<Self> {
        debug_assert!(
            self.map[self.position] != Tile::Wall
                && self.map[self.position] != Tile::Visited
                && !matches!(self.map[self.position], Tile::Door(_)),
            "we check the neighbors before visiting"
        );
        match self.map[self.position] {
            Tile::Wall | Tile::Visited | Tile::Door(_) => unreachable!("neighbor precheck fail"),
            Tile::Empty | Tile::Entrance => self.map[self.position] = Tile::Visited,
            Tile::Key(key) => {
                self.map[self.position] = Tile::Visited;
                self.map.for_each(|tile| {
                    if *tile == Tile::Door(key.to_ascii_uppercase()) || *tile == Tile::Visited {
                        *tile = Tile::Empty;
                    }
                });
                self.unclaimed_keys.remove(&key);
            }
        }
        self.steps += 1;

        let mut out = Vec::with_capacity(4);
        for direction in Direction::iter() {
            let neighbor = self.map[self.position + direction];
            match neighbor {
                Tile::Empty | Tile::Entrance | Tile::Key(_) => {
                    let mut successor = self.clone();
                    successor.position = successor.position + direction;
                    out.push(successor);
                }
                _ => {}
            }
        }
        out
    }

    fn explore_until_all_keys_claimed(self) -> usize {
        let mut queue = VecDeque::new();
        queue.push_back(self);

        while let Some(explorer) = queue.pop_front() {
            if explorer.unclaimed_keys.is_empty() {
                // subtract 1 because we visit the entrance at step 0, but it counts here as step 1
                return explorer.steps - 1;
            }
            queue.extend(explorer.visit());
        }

        unreachable!("failed to clear level")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_0() {
        let input = "
#########
#b.A.@.a#
#########
"
        .trim();
        let explorer = Explorer::from(Map::try_from(input).unwrap());
        assert_eq!(explorer.explore_until_all_keys_claimed(), 8);
    }

    #[test]
    fn example_1() {
        let input = "
########################
#f.D.E.e.C.b.A.@.a.B.c.#
######################.#
#d.....................#
########################
"
        .trim();
        let explorer = Explorer::from(Map::try_from(input).unwrap());
        assert_eq!(explorer.explore_until_all_keys_claimed(), 86);
    }
}
