use lazy_static::lazy_static;
use regex::Regex;
use std::convert::TryFrom;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Index, IndexMut, Mul, Sub};
use std::str::FromStr;

pub fn follow(traces: &[Trace]) -> Vec<Line> {
    let mut cursor = Point::new(0, 0);
    let mut out = Vec::with_capacity(traces.len());
    for trace in traces {
        let prev = cursor;
        use Direction::*;
        let (val, mul) = match trace.direction {
            Right => (&mut cursor.x, 1),
            Left => (&mut cursor.x, -1),
            Up => (&mut cursor.y, 1),
            Down => (&mut cursor.y, -1),
        };
        *val += trace.distance * mul;
        out.push(Line::new(prev, cursor));
    }
    out
}

// https://stackoverflow.com/a/1968345/504550
pub fn intersect(a: Line, b: Line) -> Option<Point> {
    let p0 = a.from;
    let p1 = a.to;
    let p2 = b.from;
    let p3 = b.to;

    let s1_x = (p1.x - p0.x) as f32;
    let s1_y = (p1.y - p0.y) as f32;
    let s2_x = (p3.x - p2.x) as f32;
    let s2_y = (p3.y - p2.y) as f32;

    let s =
        (-s1_y * (p0.x - p2.x) as f32 + s1_x * (p0.y - p2.y) as f32) / (-s2_x * s1_y + s1_x * s2_y);
    let t =
        (s2_x * (p0.y - p2.y) as f32 - s2_y * (p0.x - p2.x) as f32) / (-s2_x * s1_y + s1_x * s2_y);

    if s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0 {
        // round the results so errors line up nicely
        Some(Point::new(
            p0.x + (t * s1_x).round() as i32,
            p0.y + (t * s1_y).round() as i32,
        ))
    } else {
        None
    }
}

// bentley-ottman algorithm: http://geomalgorithms.com/a09-_intersect-3.html
// TODO: maybe finish later?
// pub fn intersections(a: &[Line], b: &[Line]) -> Vec<Point> {
//     let mut eq = Vec::with_capacity(a.len() + b.len());
//     eq.extend(a.iter().map(|l| Event {
//         line: *l,
//         color: Color::Red,
//     }));
//     eq.extend(b.iter().map(|l| Event {
//         line: *l,
//         color: Color::Black,
//     }));
//     eq.sort_unstable_by_key(|e| e.line);
//     unimplemented!()
// }

pub fn intersections_naive(ap: &[Line], bp: &[Line]) -> Vec<Point> {
    let mut isects = Vec::new();
    for a in ap {
        for b in bp {
            if let Some(isect) = intersect(*a, *b) {
                isects.push(isect);
            }
        }
    }
    isects
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}

impl Direction {
    /// (dx, dy), for Right is +x and Up is +y
    pub fn deltas(self) -> (i32, i32) {
        use Direction::*;
        match self {
            Up => (0, 1),
            Down => (0, -1),
            Right => (1, 0),
            Left => (-1, 0),
        }
    }

    pub fn turn_right(self) -> Direction {
        use Direction::*;
        match self {
            Up => Self::Right,
            Right => Self::Down,
            Down => Self::Left,
            Left => Self::Up,
        }
    }

    pub fn turn_left(self) -> Direction {
        use Direction::*;
        match self {
            Up => Self::Left,
            Left => Self::Down,
            Down => Self::Right,
            Right => Self::Up,
        }
    }

    pub fn iter() -> impl Iterator<Item = Direction> {
        use Direction::*;
        [Up, Down, Left, Right].iter().cloned()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Trace {
    direction: Direction,
    distance: i32,
}

impl FromStr for Trace {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 2 {
            return Err("len < 2".into());
        }
        let direction = match s.as_bytes()[0] {
            b'R' | b'r' => Direction::Right,
            b'L' | b'l' => Direction::Left,
            b'U' | b'u' => Direction::Up,
            b'D' | b'd' => Direction::Down,
            unknown => return Err(format!("unknown direction: {}", unknown as char)),
        };
        let distance = s[1..]
            .parse()
            .map_err(|e: std::num::ParseIntError| e.to_string())?;
        Ok(Trace {
            direction,
            distance,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    // on my machine, passing self by copy and reference are equally sized,
    // and passing by copy breaks the cleanest usage of this function in Iterator::map,
    // so I'm going to retain the reference behavior. I expect the compiler to
    // inline this function anyway.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn manhattan(&self) -> i32 {
        self.x.abs() + self.y.abs()
    }

    pub fn abs(&self) -> Point {
        Point {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }
}

impl From<(usize, usize)> for Point {
    fn from((x, y): (usize, usize)) -> Self {
        Self::new(
            i32::try_from(x).unwrap_or(i32::MAX),
            i32::try_from(y).unwrap_or(i32::MAX),
        )
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Add<(i32, i32)> for Point {
    type Output = Point;

    fn add(self, (dx, dy): (i32, i32)) -> Point {
        Point {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

impl Add<Direction> for Point {
    type Output = Point;

    fn add(self, direction: Direction) -> Point {
        self + direction.deltas()
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<i32> for Point {
    type Output = Point;

    fn mul(self, other: i32) -> Point {
        Point {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl Div<i32> for Point {
    type Output = Point;

    fn div(self, other: i32) -> Point {
        Point {
            x: self.x / other,
            y: self.y / other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Line {
    pub from: Point,
    pub to: Point,
}

impl Line {
    pub fn new(from: Point, to: Point) -> Line {
        Line { from, to }
    }

    pub fn manhattan_len(&self) -> i32 {
        (self.to - self.from).manhattan()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Vector3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vector3 {
    pub fn new(x: i32, y: i32, z: i32) -> Vector3 {
        Vector3 { x, y, z }
    }

    pub fn abs_sum(&self) -> i32 {
        self.x.abs() + self.y.abs() + self.z.abs()
    }
}

lazy_static! {
    static ref VEC3_RE: Regex = Regex::new(
        r"(?i)<\s*(x=\s*)?(?P<x>-?\d+),\s*(y=\s*)?(?P<y>-?\d+),\s*(z=\s*)?(?P<z>-?\d+)\s*>"
    )
    .unwrap();
}

impl FromStr for Vector3 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let captures = VEC3_RE.captures(s).ok_or(format!("no regex match"))?;
        Ok(Vector3 {
            x: captures
                .name("x")
                .unwrap()
                .as_str()
                .parse()
                .map_err(|err| format!("x: {}", err))?,
            y: captures
                .name("y")
                .unwrap()
                .as_str()
                .parse()
                .map_err(|err| format!("y: {}", err))?,
            z: captures
                .name("z")
                .unwrap()
                .as_str()
                .parse()
                .map_err(|err| format!("z: {}", err))?,
        })
    }
}

impl fmt::Display for Vector3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<x={:3}, y={:3}, z={:3}>", self.x, self.y, self.z)
    }
}

impl AddAssign for Vector3 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl Add for Vector3 {
    type Output = Vector3;

    fn add(self, other: Self) -> Self {
        Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

/// A Map keeps track of a tile grid.
///
/// It is based on immutable data structures, so is cheap to clone.
///
/// Its coordinate system assumes that the origin is in the lower left,
/// for compatibility with Direction.
#[derive(Clone, Default)]
pub struct Map<T: Clone> {
    tiles: im::Vector<T>,
    width: usize,
    height: usize,
}

impl<T: Clone + Default> Map<T> {
    pub fn new(width: usize, height: usize) -> Map<T> {
        Map {
            tiles: vec![T::default(); width * height].into(),
            width,
            height,
        }
    }
}

impl<T, R> From<&[R]> for Map<T>
where
    T: Clone,
    R: AsRef<[T]>,
{
    /// Convert an input 2d array into a map.
    ///
    /// Note that the input array must already be arranged with the y axis
    /// as the outer array and the orientation such that source[0][0] is the
    /// lower left corner of the map.
    ///
    /// Panics if the input array is not rectangular.
    fn from(source: &[R]) -> Map<T> {
        let height = source.len();
        if height == 0 {
            return Map {
                tiles: im::Vector::new(),
                width: 0,
                height: 0,
            };
        }

        let width = source[0].as_ref().len();
        assert!(
            source
                .as_ref()
                .iter()
                .all(|row| row.as_ref().len() == width),
            "input must be rectangular"
        );

        let mut tiles = im::Vector::new();
        for row in source.iter() {
            for tile in row.as_ref().iter() {
                tiles.push_back(tile.clone());
            }
        }

        Map { tiles, width, height }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapConversionErr<T>
where
    T: TryFrom<char>,
    <T as TryFrom<char>>::Error: std::fmt::Debug + Clone + PartialEq + Eq,
{
    TileConversion(<T as TryFrom<char>>::Error),
    NotRectangular,
}

impl<T> fmt::Display for MapConversionErr<T>
where
    T: TryFrom<char>,
    <T as TryFrom<char>>::Error: std::fmt::Debug + Clone + PartialEq + Eq,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::TileConversion(err) => write!(f, "{:?}", err),
            Self::NotRectangular => write!(f, "maps must be rectangular"),
        }
    }
}

impl<T> std::error::Error for MapConversionErr<T>
where
    T: fmt::Debug + TryFrom<char>,
    <T as TryFrom<char>>::Error: std::fmt::Debug + Clone + PartialEq + Eq,
{
}

impl<T> Map<T>
where
    T: Clone + TryFrom<char>,
    <T as TryFrom<char>>::Error: std::fmt::Debug + Clone + PartialEq + Eq,
{
    // we actually impl<T, R> TryFrom<R> for Map<T> because there's a
    // coherence conflict with the stdlib blanket impl
    //
    //   impl<T, U> std::convert::TryFrom<U> for T where U: std::convert::Into<T>;
    //
    // Because there's a chance that R also implements Into<Map<T>>, we can't do it.
    //
    // That doesn't stop us from doing it here, and implementing the official trait for
    // a few concrete types
    fn try_from<R>(input: R) -> Result<Self, MapConversionErr<T>>
    where
        R: std::io::BufRead,
    {
        let mut arr = Vec::new();

        for line in input.lines() {
            let line = line.unwrap();

            let mut row = Vec::with_capacity(line.len());
            for ch in line.chars() {
                row.push(T::try_from(ch).map_err(MapConversionErr::TileConversion)?);
            }
            if !row.is_empty() {
                arr.push(row);
            }
        }

        if !arr.is_empty() {
            let width = arr[0].len();
            if !arr.iter().all(|row| row.len() == width) {
                Err(MapConversionErr::NotRectangular)?;
            }
        }

        // shift the origin
        arr.reverse();

        Ok(Map::from(arr.as_slice()))
    }
}

impl<T> TryFrom<&str> for Map<T>
where
    T: Clone + TryFrom<char>,
    <T as TryFrom<char>>::Error: std::fmt::Debug + Clone + PartialEq + Eq,
{
    type Error = MapConversionErr<T>;

    /// the input should be in natural graphical order:
    /// its first characters are the top left.
    fn try_from(input: &str) -> Result<Self, Self::Error> {
        <Self>::try_from(input.as_bytes())
    }
}

impl<T> TryFrom<std::fs::File> for Map<T>
where
    T: Clone + TryFrom<char>,
    <T as TryFrom<char>>::Error: std::fmt::Debug + Clone + PartialEq + Eq,
{
    type Error = MapConversionErr<T>;

    /// the input should be in natural graphical order:
    /// its first characters are the top left.
    fn try_from(input: std::fs::File) -> Result<Self, Self::Error> {
        <Self>::try_from(std::io::BufReader::new(input))
    }
}

impl<T> TryFrom<&std::path::Path> for Map<T>
where
    T: 'static + fmt::Debug + Clone + TryFrom<char>,
    <T as TryFrom<char>>::Error: Send + Sync + std::fmt::Debug + Clone + PartialEq + Eq,
{
    type Error = std::io::Error;

    /// the input should be in natural graphical order:
    /// its first characters are the top left.
    fn try_from(path: &std::path::Path) -> Result<Self, Self::Error> {
        <Self as TryFrom<std::fs::File>>::try_from(std::fs::File::open(path)?)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, Box::new(e)))
    }
}

impl<T: Clone> Index<(usize, usize)> for Map<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &T {
        self.tiles.index(x + (y * self.width))
    }
}

impl<T: Clone> Index<Point> for Map<T> {
    type Output = T;

    /// Panics if point.x or point.y < 0
    fn index(&self, point: Point) -> &T {
        assert!(
            point.x >= 0 && point.y >= 0,
            "point must be in the positive quadrant"
        );
        self.index((point.x as usize, point.y as usize))
    }
}

impl<T: Clone> IndexMut<(usize, usize)> for Map<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut T {
        self.tiles.index_mut(x + (y * self.width))
    }
}

impl<T: Clone> IndexMut<Point> for Map<T> {
    /// Panics if point.x or point.y < 0
    fn index_mut(&mut self, point: Point) -> &mut T {
        assert!(
            point.x >= 0 && point.y >= 0,
            "point must be in the positive quadrant"
        );
        self.index_mut((point.x as usize, point.y as usize))
    }
}

impl<T: Clone + Into<char>> fmt::Display for Map<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                write!(f, "{}", self[(x, y)].clone().into())?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl<T: Clone> Map<T> {
    pub fn for_each<F>(&mut self, update: F)
    where
        F: FnMut(&mut T),
    {
        self.tiles.iter_mut().for_each(update);
    }

    pub fn for_each_point<F>(&mut self, mut update: F)
    where
        F: FnMut(&mut T, Point),
    {
        for y in 0..self.height {
            for x in 0..self.width {
                update(self.index_mut((x, y)), (x, y).into());
            }
        }
    }
}
