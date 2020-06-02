use crate::{
    geometry::{Direction, Point},
    Exercise,
};
use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;

pub struct Day;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let explorer = Explorer::from_input(path);
        println!("steps to claim all keys: {}", explorer.explore_until_all_keys_claimed());
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
}

#[derive(Default, Clone)]
struct Explorer {
    map: Vec<Vec<Tile>>,
    entrance: Point,
    position: Point,
    steps: usize,
    unclaimed_keys: HashSet<char>,
}

impl Explorer {

    fn from_buffered<B: BufRead>(reader: B) -> Explorer {
        let mut explorer = Explorer::default();

        for (y, line) in reader.lines().enumerate() {
            let line = line.unwrap();
            if line.is_empty() {
                continue;
            }

            let mut row = Vec::with_capacity(line.len());
            for (x, ch) in line.chars().enumerate() {
                match ch {
                    '#' => row.push(Tile::Wall),
                    '.' => row.push(Tile::Empty),
                    'A'..='Z' => row.push(Tile::Door(ch)),
                    'a'..='z' => {
                        row.push(Tile::Key(ch));
                        explorer.unclaimed_keys.insert(ch);
                    }
                    '@' => {
                        row.push(Tile::Empty);
                        explorer.entrance = Point::from((x, y));
                        explorer.position = explorer.entrance;
                    }
                    _ => unreachable!("invalid input map"),
                }
            }
            explorer.map.push(row);
        }

        explorer
    }

    fn from_input(path: &Path) -> Explorer {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        Self::from_buffered(reader)
    }

    #[cfg(test)]
    fn from_str(input: &str) -> Explorer {
        Self::from_buffered(input.as_bytes())
    }

    fn tile(&self) -> Tile {
        self.map[self.position.y as usize][self.position.x as usize]
    }

    fn tile_mut(&mut self) -> &mut Tile {
        &mut self.map[self.position.y as usize][self.position.x as usize]
    }

    fn tiles<F>(&mut self, update: F) where F: Fn(&mut Tile) {
        for row in self.map.iter_mut() {
            for tile in row.iter_mut() {
                update(tile);
            }
        }
    }

    fn visit(mut self) -> Vec<Self> {
        match self.tile() {
            Tile::Wall | Tile::Visited | Tile::Door(_) => {
                return Vec::new();
            }
            Tile::Empty => *self.tile_mut() = Tile::Visited,
            Tile::Key(key) => {
                *self.tile_mut() = Tile::Visited;
                self.tiles(|tile| if *tile == Tile::Door(key) || *tile == Tile::Visited {
                    *tile = Tile::Empty;
                });
                self.unclaimed_keys.remove(&key);
            }
        }
        self.steps += 1;

        let mut out = Vec::with_capacity(4);
        for direction in Direction::iter() {
            let mut successor = self.clone();
            successor.position = successor.position + direction;
            out.push(successor);
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
".trim();
        let explorer = Explorer::from_str(input);
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
".trim();
        let explorer = Explorer::from_str(input);
        assert_eq!(explorer.explore_until_all_keys_claimed(), 86);
    }
}