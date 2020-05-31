use crate::{
    ddbg,
    geometry::{Direction, Point},
    intcode::{channel, Intcode, IntcodeMemory, Word},
    parse, CommaSep, Exercise,
};
use crossbeam_channel::{Receiver, Sender};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::path::Path;
use std::thread;

const MAP_DIMENSION: usize = 128;

fn out_of_bounds(point: Point) -> bool {
    point.x < 0 || point.y < 0 || point.x >= MAP_DIMENSION as i32 || point.y >= MAP_DIMENSION as i32
}

pub struct Day;

impl Day {
    fn find_target_with_droid(path: &Path) -> Droid {
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
        droid
    }
}

impl Exercise for Day {
    fn part1(&self, path: &Path) {
        let mut droid = Self::find_target_with_droid(path);
        #[cfg(feature = "debug")]
        {
            println!("target location: {:?}", droid.position);
            println!("{}", droid.show_map());
            println!("route to origin: {:#?}", droid.navigate_to(droid.origin));
        }
        let oxygenator = droid.position;
        // it turns out to be _much_ faster to intentionally fill the map
        // first, than to fill it by accident via repeated failing application
        // of A*.
        droid.fill_map();
        droid.proceed_to(oxygenator);
        let shortest_path_len = droid.find_shortest_path_to_origin().len();
        println!("{}", droid.show_map());
        println!("shortest path to o2 system: {}", shortest_path_len);
    }

    fn part2(&self, path: &Path) {
        let mut droid = Self::find_target_with_droid(path);
        let oxygenator = droid.position;
        // at this point, the droid has a very partial and incomplete understanding
        // of the map. Let's fill in the unknown-but-reachable areas.
        //
        // ... it may prove faster to greedily fill all unknown areas first,
        // then start navigating back and forth between the origin and
        // target, than it was to repeatedly A* and abort on a surprise wall
        // back in part 1. That can be a TODO for later.
        droid.fill_map();
        droid.proceed_to(oxygenator);

        // we have to start at -1 because the origin tile should fill at time 0; if this
        // is initialized at 0, it fills at time 1.
        let mut minutes = -1;

        enum QueueItem {
            TimePasses,
            Position(Point),
        }
        use QueueItem::*;

        let mut queue = VecDeque::new();
        queue.push_back(Position(droid.position));
        queue.push_back(TimePasses);

        while let Some(qi) = queue.pop_front() {
            match qi {
                Position(position) => match droid.tile(position).unwrap() {
                    MapTile::Unknown => unreachable!("we're in an explored maze"),
                    MapTile::Empty => {
                        *droid.tile_mut(position).unwrap() = MapTile::Oxygen;
                        for direction in Direction::iter() {
                            queue.push_back(Position(position + direction.deltas()));
                        }
                    }
                    MapTile::Wall | MapTile::Oxygen => {
                        // no need to retrace our steps
                    }
                },
                TimePasses => {
                    queue.push_back(TimePasses);
                    if let Some(TimePasses) = queue.front() {
                        break;
                    }
                    minutes += 1;
                }
            }
        }

        println!("{}", droid.show_map());
        println!("{} minutes before o2 saturation", minutes);
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
            (Some(MapTile::Empty), Status::HitWall) | (Some(MapTile::Oxygen), Status::HitWall) => {
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

    fn find_nearest_reachable_unknown(&self) -> Option<Point> {
        // first things first, make a copy of the map to work on. We're going to
        // be marking previously-visited tiles as oxygenated, because it's a
        // convenient label to use, but we don't want to actually edit the
        // internal map at all.
        let mut map = self.map.clone();

        // we need a queue of tiles to visit
        let mut queue = VecDeque::new();
        queue.push_back(self.position);

        // now, we can start flood-filling the map with oxygen representing
        // discovered areas. As soon as we find a reachable unknown area,
        // we break out of the discovery loop and navigate the physical
        // robot to it, then restart the process.
        while let Some(position) = queue.pop_front() {
            match map[position.y as usize][position.x as usize] {
                MapTile::Unknown => {
                    return Some(position);
                }
                MapTile::Empty => {
                    map[position.y as usize][position.x as usize] = MapTile::Oxygen;
                    for direction in Direction::iter() {
                        queue.push_back(position + direction.deltas());
                    }
                }
                MapTile::Oxygen | MapTile::Wall => {
                    // nop: we can't visit walls, and we shouldn't revisit
                    // oxygenated tiles, so they just get popped off
                    // the queue.
                }
            }
        }

        // by draining the queue, we prove that there are no more reachable
        // unknown tiles. hooray!
        None
    }

    /// navigate to the oxygen system.
    fn find_target(&mut self) -> Option<()> {
        // general strategy:
        // 1. find the nearest (by travel distance) unknown tile
        // 2. go there
        // 3. is it the target?
        // 4. if not: repeat
        loop {
            let nearest_unknown = self.find_nearest_unknown()?;
            let path = self.navigate_to(nearest_unknown)?;
            for direction in path {
                let status = self.go(direction);
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
    ///
    /// panics if the given target is a wall
    fn proceed_to(&mut self, target: Point) -> bool {
        let mut attempt = 0_usize;

        while self.position != target {
            for step in self.navigate_to(target).unwrap() {
                if self.go(step) == Status::HitWall {
                    attempt += 1;
                    break;
                }
            }
            #[cfg(feature = "debug")]
            {
                if attempt > 0 && attempt & 0x3f == 0 {
                    dbg!(attempt);
                }
                if attempt > 0 && attempt & 0xff == 0 {
                    println!("attempt {} attempts to get back and forth:", attempt);
                    println!("{}", self.show_map());
                }
            }
        }

        debug_assert_eq!(self.position, target, "failed to reach desired point!");
        #[cfg(feature = "debug")]
        println!("reached target: {:?}", target);
        attempt == 0
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

        // we need to successfully traverse the maze at least
        // twice in a row before we can assume that subsequent
        // attempts won't discover any new surprising walls.
        // Instead of hard-coding this with the assumption that
        // two is all that is required, we're going to use a
        // bitfield, so if we need to change that number, we can.
        let mut bitfield: u8 = 0;
        const SUCCESSES: u8 = 0b11;
        while bitfield & SUCCESSES != SUCCESSES {
            let succeeded = if ddbg!(self.proceed_to(destination)) {
                1
            } else {
                0
            };
            bitfield = (bitfield << 1) | succeeded;
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

    fn show_map(&self) -> String {
        let mut min_x = usize::MAX;
        let mut min_y = usize::MAX;
        let mut max_x = 0;
        let mut max_y = 0;

        for (y, row) in self.map.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if *tile != MapTile::Unknown {
                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }
        }

        // from the bounds, we know the capacity we need
        // note that we add a column: newlines at the end of each row
        let capacity = (1 + max_x - min_x) * (max_y - min_y);
        let mut out = String::with_capacity(capacity);

        // iterate the rows backwards: in AoC, the origin is at the lower
        // left corner of the map
        for (y, row) in self.map[min_y..=max_y].iter().enumerate().rev() {
            for (x, tile) in row[min_x..=max_x].iter().enumerate() {
                if self.position == Point::new((x + min_x) as i32, (y + min_y) as i32) {
                    out.push('D');
                    continue;
                }
                if self.origin == Point::from((x + min_x, y + min_y)) {
                    out.push('O');
                    continue;
                }
                out.push(match tile {
                    MapTile::Unknown => ' ',
                    MapTile::Wall => '#',
                    MapTile::Empty => '.',
                    MapTile::Oxygen => 'x',
                });
            }
            out.push('\n');
        }
        out
    }

    /// discover all reachable tiles
    fn fill_map(&mut self) {
        // it's non-trivial to discover all reachable tiles: we can't just keep
        // attempting to pathfind to the nearest unreachable one, because an
        // unknown but large quantity of unreachable ones will be beyond the
        // bounds of the map.
        //
        // Instead, we'll take this strategy: in memory, flood-fill the map
        // from the current position. The first time we discover an unknown
        // reachable tile, halt and navigate to that tile. Then repeat until no
        // unknown reachable tiles exist.

        while let Some(position) = self.find_nearest_reachable_unknown() {
            // find_nearest_reachable_unknown just follows traversable steps,
            // so it must always be true that we can navigate to a point it returned
            let path = self.navigate_to(position).unwrap();
            for step in path {
                if let Status::HitWall = self.go(step) {
                    // if we hit a wall on the way to our destination, that's
                    // fine: let's just keep finding more unknown points
                    break;
                }
            }
        }
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
    Oxygen,
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
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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
