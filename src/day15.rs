use crate::{
    ddbg,
    geometry::{Direction, Map as GenericMap, Point, Traversable},
    intcode::{channel, Intcode, IntcodeMemory, Word},
    parse, CommaSep, Exercise,
};
use crossbeam_channel::{Receiver, Sender};
use std::cmp::Ordering;
use std::collections::{VecDeque};
use std::path::Path;
use std::thread;

const MAP_DIMENSION: usize = 128;

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
            println!("route to origin: {:#?}", droid.map.navigate(droid.position, droid.origin));
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
                Position(position) => match droid.map[position] {
                    MapTile::Unknown => unreachable!("we're in an explored maze"),
                    MapTile::Empty => {
                        droid.map[position] = MapTile::Oxygen;
                        for direction in Direction::iter() {
                            queue.push_back(Position(position + direction));
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
    map: Map,
    origin: Point,
    position: Point,
    controller: Sender<i64>,
    sensor: Receiver<i64>,
}

impl Droid {
    fn new(controller: Sender<i64>, sensor: Receiver<i64>) -> Self {
        let origin = Point::new((MAP_DIMENSION / 2) as i32, (MAP_DIMENSION / 2) as i32);
        let mut droid = Droid {
            map: Map::new(MAP_DIMENSION, MAP_DIMENSION),
            origin,
            position: origin,
            controller,
            sensor,
        };
        droid.map[droid.position] = MapTile::Empty;
        droid
    }


    fn go(&mut self, direction: Direction) -> Status {
        self.controller.send(movement_command(direction)).unwrap();
        let status: Status = self.sensor.recv().unwrap().into();
        let destination_tile = self.position + direction.deltas();
        match (self.map[destination_tile], status) {
            (MapTile::Wall, _) => {
                unreachable!("should never intentionally drive into a wall")
            }
            (MapTile::Empty, Status::HitWall) | (MapTile::Oxygen, Status::HitWall) => {
                unreachable!("unreliable cartography! aborting")
            }
            (MapTile::Unknown, Status::HitWall) => {
                self.map[destination_tile] = MapTile::Wall;
            }
            (_, Status::Moved) | (_, Status::FoundTarget) => {
                self.position = destination_tile;
                self.map[self.position] = MapTile::Empty;
            }
        }
        status
    }

    fn find_nearest_reachable_unknown(&self) -> Option<Point> {
        let mut unknown = None;
        self.map.reachable_from(self.position, |tile, position| {
            if *tile == MapTile::Unknown {
                unknown = Some(position);
                true
            } else {
                false
            }
        });
        unknown
    }

    /// navigate to the oxygen system.
    fn find_target(&mut self) -> Option<()> {
        // general strategy:
        // 1. find the nearest (by travel distance) unknown tile
        // 2. go there
        // 3. is it the target?
        // 4. if not: repeat
        loop {
            let nearest_unknown = self.find_nearest_reachable_unknown()?;
            let path = self.map.navigate(self.position, nearest_unknown)?;
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
            for step in self.map.navigate(self.position, target).unwrap() {
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
        self.map.navigate(self.position, self.origin).unwrap()
    }

    fn show_map(&self) -> String {
        let mut min_x = usize::MAX;
        let mut min_y = usize::MAX;
        let mut max_x = 0;
        let mut max_y = 0;

        self.map.for_each_point(|tile, point| {
            if *tile != MapTile::Unknown {
                min_x = min_x.min(point.x as usize);
                min_y = min_y.min(point.y as usize);
                max_x = max_x.max(point.x as usize);
                max_y = max_y.max(point.y as usize);
            }
        });

        // from the bounds, we know the capacity we need
        // note that we add a column: newlines at the end of each row
        let capacity = (1 + max_x - min_x) * (max_y - min_y);
        let mut out = String::with_capacity(capacity);

        // iterate the rows backwards: in AoC, the origin is at the lower
        // left corner of the map
        for y in (min_y..=max_y).rev() {
            for x in min_x..=max_x {
                let point = Point::from((x, y));
                if point == self.position {
                    out.push('D');
                    continue;
                }
                if point == self.origin {
                    out.push('O');
                    continue;
                }
                out.push(match self.map[point] {
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
            let path = self.map.navigate(self.position, position).unwrap();
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

type Map = GenericMap<MapTile>;

impl Default for MapTile {
    fn default() -> Self {
        MapTile::Unknown
    }
}

impl From<MapTile> for Traversable {
    fn from(tile: MapTile) -> Traversable {
        use MapTile::*;
        use Traversable::*;
        match tile {
            Unknown => Free,
            Empty => Free,
            Wall => Obstructed,
            Oxygen => Free,
        }
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
