use crate::{
    ddbg,
    geometry::{Direction, Point},
    intcode::{channel, Intcode, IntcodeMemory, Word},
    parse, CommaSep, Exercise,
};
use crossbeam_channel::{Receiver, Sender};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::path::Path;
use std::thread;

const MAP_DIMENSION: usize = 1024;

fn out_of_bounds(point: Point) -> bool {
    point.x < 0 || point.y < 0 || point.x >= MAP_DIMENSION as i32 || point.y >= MAP_DIMENSION as i32
}

pub struct Day;

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let memory: IntcodeMemory = parse::<CommaSep<Word>>(path).unwrap().flatten().collect();
        let (controller, inputs) = channel();
        let (outputs, sensor) = channel();

        let mut computer = Intcode::new(memory)
            .with_inputs(inputs)
            .with_outputs(outputs);
        thread::spawn(move || {
            computer.run().unwrap();
        });

        let mut droid = Droid::new(controller, sensor);
        droid.find_target();
        #[cfg(feature="debug")]
        println!("target location: {:?}", droid.position);
        println!(
            "shortest path to o2 system: {}",
            droid.find_shortest_path_to_origin().len(),
        );
    }

    fn part2(&self, _path: &Path) {
        unimplemented!()
    }
}

struct Droid {
    map: [[MapTile; MAP_DIMENSION]; MAP_DIMENSION],
    origin: Point,
    position: Point,
    controller: Sender<i64>,
    sensor: Receiver<i64>,
}

impl Droid {
    fn new(controller: Sender<i64>, sensor: Receiver<i64>) -> Self {
        let origin = Point::new((MAP_DIMENSION / 2) as i32, (MAP_DIMENSION / 2) as i32);
        let mut droid = Droid {
            map: [[MapTile::default(); MAP_DIMENSION]; MAP_DIMENSION],
            origin,
            position: origin,
            controller,
            sensor,
        };
        *droid.tile_mut(droid.position).unwrap() = MapTile::Empty;
        droid
    }

    fn tile(&self, point: Point) -> Option<&MapTile> {
        if out_of_bounds(point) {
            return None;
        }
        Some(&self.map[point.y as usize][point.x as usize])
    }

    fn tile_mut(&mut self, point: Point) -> Option<&mut MapTile> {
        if out_of_bounds(point) {
            return None;
        }
        Some(&mut self.map[point.y as usize][point.x as usize])
    }

    fn go(&mut self, direction: Direction) -> Status {
        self.controller.send(movement_command(direction)).unwrap();
        let status: Status = self.sensor.recv().unwrap().into();
        let destination_tile = self.position + direction.deltas();
        match (self.tile(destination_tile), status) {
            (None, _) => unreachable!("should never drive off the map"),
            (Some(MapTile::Wall), _) => {
                unreachable!("should never intentionally drive into a wall")
            }
            (Some(MapTile::Empty), Status::HitWall) => {
                unreachable!("unreliable cartography! aborting")
            }
            (Some(MapTile::Unknown), Status::HitWall) => {
                *self.tile_mut(destination_tile).unwrap() = MapTile::Wall;
            }
            (_, Status::Moved) | (_, Status::FoundTarget) => {
                self.position = destination_tile;
                *self.tile_mut(self.position).unwrap() = MapTile::Empty;
            }
        }
        status
    }

    /// navigate to the target point using A*
    // https://en.wikipedia.org/wiki/A*_search_algorithm#Pseudocode
    fn navigate_to(&self, target: Point) -> Option<Vec<Direction>> {
        let initial = AStarNode {
            cost: 0,
            position: self.position,
        };
        let mut open_set = BinaryHeap::new();
        open_set.push(initial);

        // key: node
        // value: node preceding it on the cheapest known path from start
        let mut came_from = HashMap::new();

        // gscore
        // key: position
        // value: cost of cheapest path from start to node
        let mut cheapest_path_cost = HashMap::new();
        cheapest_path_cost.insert(self.position, 0_u32);

        // fscore
        // key: position
        // value: best guess as to total cost from here to finish
        let mut total_cost_guess = HashMap::new();
        total_cost_guess.insert(self.position, (target - self.position).manhattan() as u32);

        while let Some(AStarNode { cost, position }) = open_set.pop() {
            if position == target {
                let mut current = position;
                let mut path = Vec::new();
                while let Some((direction, predecessor)) = came_from.remove(&current) {
                    current = predecessor;
                    path.push(direction);
                }
                path.reverse();
                return Some(path);
            }

            for direction in Direction::iter() {
                let neighbor = position + direction.deltas();
                match self.tile(neighbor) {
                    Some(MapTile::Wall) => {} // walls aren't neighbors; continue
                    None => {}                // don't explore off the map; continue
                    Some(_) => {
                        let tentative_cheapest_path_cost = cost + 1;
                        if tentative_cheapest_path_cost
                            < cheapest_path_cost
                                .get(&neighbor)
                                .cloned()
                                .unwrap_or(u32::MAX)
                        {
                            // this path to the neighbor is better than any previous one
                            came_from.insert(neighbor, (direction, position));
                            cheapest_path_cost.insert(neighbor, tentative_cheapest_path_cost);
                            total_cost_guess.insert(
                                neighbor,
                                tentative_cheapest_path_cost
                                    + (target - neighbor).manhattan() as u32,
                            );

                            // this thing with the iterator is not very efficient, but for some weird reason BinaryHeap
                            // doesn't have a .contains method; see
                            // https://github.com/rust-lang/rust/issues/66724
                            if open_set
                                .iter()
                                .find(|elem| elem.position == neighbor)
                                .is_none()
                            {
                                open_set.push(AStarNode {
                                    cost: tentative_cheapest_path_cost,
                                    position: neighbor,
                                });
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn spiral_out_from_current_position(&self) -> impl Iterator<Item = Point> {
        let mut direction = Direction::Up;
        let mut steps = 0;
        let mut edge_len = 1;
        let mut second = false;
        std::iter::successors(Some(self.position), move |prev| {
            let next = *prev + direction.deltas();
            steps += 1;
            if steps == edge_len {
                direction = direction.turn_right();
                steps = 0;
                if second {
                    edge_len += 1;
                }
                second = !second;
            }
            Some(next)
        })
        // in the worst case, where we start in the bottom right corner
        // and the last unknown tile is in the top right corner,
        // we have to examine 4 * the total dimensions of the map to be sure we've inspected every possible tile
        .take(MAP_DIMENSION * MAP_DIMENSION * 4)
        .filter(|point| !out_of_bounds(*point))
    }

    fn find_nearest_unknown(&self) -> Option<Point> {
        self.spiral_out_from_current_position()
            .map(|point| (point, self.tile(point)))
            .find(|(_, tile)| tile.cloned() == Some(MapTile::Unknown))
            .map(|(point, _)| point)
    }

    /// navigate to the oxygen system.
    fn find_target(&mut self) -> Option<()> {
        // general strategy:
        // 1. find the nearest (by travel distance) unknown tile
        // 2. go there
        // 3. is it the target?
        // 4. if not: repeat
        loop {
            ddbg!(self.position, self.tile(self.position));
            let nearest_unknown = self.find_nearest_unknown()?;
            ddbg!(nearest_unknown);
            let path = self.navigate_to(nearest_unknown)?;
            ddbg!(&path);
            for direction in path {
                ddbg!(direction);
                let status = self.go(direction);
                ddbg!(status, self.position);
                match status {
                    Status::FoundTarget => return Some(()),
                    Status::HitWall => break, // reevaluate where the nearest empty spaces are
                    Status::Moved => {}
                }
            }
        }
    }

    /// navigate to the target point, recomputing when unexpected walls appear.
    /// returns true if the original plan succeeded in guiding the robot to
    /// the destination, false if recomputation was required.
    ///
    /// will infinitely loop if the given target is unreachable from the initial position.
    fn proceed_to(&mut self, target: Point) -> bool {
        let mut first_plan_worked = true;

        while self.position != target {
            for step in dbg!(self.navigate_to(target).unwrap()) {
                if dbg!(self.go(dbg!(step))) == Status::HitWall {
                    first_plan_worked = false;
                    break;
                }
            }
        }

        debug_assert_eq!(self.position, target, "failed to reach desired point!");
        first_plan_worked
    }

    /// finds the shortest path from the current position to the origin
    ///
    /// Repeatedly shuttles between the current position and the origin,
    /// terminating when the path length is unchanged. This works because
    /// our navigation function treats unknown spaces as unoccupied, so
    /// paths through unknown spaces are preferred.
    ///
    /// On termination, the robot will be in the same position it began in.
    fn find_shortest_path_to_origin(&mut self) -> Vec<Direction> {
        let initial_position = self.position;

        let mut origin = self.position;
        let mut destination = self.origin;
        while !self.proceed_to(destination) {
            std::mem::swap(&mut origin, &mut destination);
        }

        if self.position != initial_position {
            assert!(
                self.proceed_to(initial_position),
                "we must have found the shortest path by now"
            );
        }

        debug_assert_eq!(
            initial_position, self.position,
            "postcondition wasn't upheld!"
        );
        self.navigate_to(self.origin).unwrap()
    }
}

fn movement_command(direction: Direction) -> i64 {
    match direction {
        Direction::Up => 1,
        Direction::Down => 2,
        Direction::Left => 3,
        Direction::Right => 4,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MapTile {
    Unknown,
    Empty,
    Wall,
}

impl Default for MapTile {
    fn default() -> Self {
        MapTile::Unknown
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Status {
    HitWall,
    Moved,
    FoundTarget,
}

impl From<i64> for Status {
    fn from(n: i64) -> Self {
        match n {
            0 => Self::HitWall,
            1 => Self::Moved,
            2 => Self::FoundTarget,
            _ => unreachable!("received unexpected sensor status"),
        }
    }
}

/// A* State
// https://doc.rust-lang.org/std/collections/binary_heap/#examples
#[derive(Copy, Clone, Eq, PartialEq)]
struct AStarNode {
    cost: u32,
    position: Point,
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for AStarNode {
    fn cmp(&self, other: &AStarNode) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.position.cmp(&other.position))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &AStarNode) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
